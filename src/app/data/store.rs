use std::sync::{Arc, Mutex};
use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};
use rusqlite::{Connection, params};
use eframe::egui::plot::Value;
use crate::app::data::{Panel, Source};

use super::FetchError;

pub trait DataStorage {
	fn add_panel(&self, name:&str);
}

pub struct SQLiteDataStore {
	conn: Connection,
	pub(crate) panels: Mutex<Vec<Panel>>,
}

impl SQLiteDataStore {
	pub fn new(path: std::path::PathBuf) -> Result<Self, rusqlite::Error> {
		let conn = Connection::open(path)?;
		
		conn.execute(
			"CREATE TABLE IF NOT EXISTS panels (
				id INTEGER PRIMARY KEY,
				name TEXT UNIQUE,
				view_scroll BOOL,
				view_size INT,
				width INT,
				height INT
			);",
			[],
		)?;

		conn.execute(
			"CREATE TABLE IF NOT EXISTS sources (
				id INTEGER PRIMARY KEY,
				name TEXT UNIQUE,
				url TEXT,
				interval INT,
				query_x TEXT,
				query_y TEXT,
				panel_id INT
			);",
			[],
		)?;

		conn.execute(
			"CREATE TABLE IF NOT EXISTS points (
				id INTEGER PRIMARY KEY,
				panel_id INT,
				source_id INT,
				x FLOAT,
				y FLOAT
			);",
			[],
		)?;

		let mut store = SQLiteDataStore { 
			conn,
			panels: Mutex::new(Vec::new()),
		};

		store.load_panels()?;

		return Ok(store);
	}

	fn load_values(&self, panel_id:i32, source_id:i32) -> rusqlite::Result<Vec<Value>> {
		let mut values : Vec<Value> = Vec::new();
		let mut statement = self.conn.prepare("SELECT x, y FROM points WHERE panel_id = ? AND source_id = ?")?;
		let values_iter = statement.query_map(params![panel_id, source_id], |row| {
			Ok(Value{ x: row.get(0)?, y: row.get(1)? })
		})?;

		for value in values_iter {
			if let Ok(v) = value {
				values.push(v);
			}
		}

		Ok(values)
	}

	fn put_value(&self, panel_id:i32, source_id:i32, v:Value) -> rusqlite::Result<usize> {
		self.conn.execute(
			"INSERT INTO points(panel_id, source_id, x, y) VALUES (?, ?, ?, ?)",
			params![panel_id, source_id, v.x, v.y],
		)
	}

	fn load_sources(&self, panel_id:i32) -> rusqlite::Result<Vec<Source>> {
		let mut sources : Vec<Source> = Vec::new();
		let mut statement = self.conn.prepare("SELECT * FROM sources WHERE panel_id = ?")?;
		let sources_iter = statement.query_map(params![panel_id], |row| {
			Ok(Source{
				id: row.get(0)?,
				name: row.get(1)?,
				url: row.get(2)?,
				interval: row.get(3)?,
				last_fetch: Utc.ymd(1970, 1, 1).and_hms(0, 0, 0),
				query_x: row.get(4)?,
				// compiled_query_x: Arc::new(Mutex::new(jq_rs::compile(row.get::<usize, String>(4)?.as_str()).unwrap())),
				query_y: row.get(5)?,
				// compiled_query_y: Arc::new(Mutex::new(jq_rs::compile(row.get::<usize, String>(5)?.as_str()).unwrap())),
				panel_id: row.get(6)?,
				data: Mutex::new(Vec::new()),
			})
		})?;

		for source in sources_iter {
			if let Ok(mut s) = source {
				s.data = Mutex::new(self.load_values(panel_id, s.id)?);
				sources.push(s);
			}
		}

		Ok(sources)
	}

	fn put_source(&self, panel_id:i32, s:Source) -> rusqlite::Result<usize> {
		self.conn.execute(
			"INSERT INTO sources(id, name, url, interval, query_x, query_y, panel_id) VALUES (?, ?, ?, ?, ?, ?, ?)",
			params![s.id, s.name, s.url, s.interval, s.query_x, s.query_y, panel_id],
		)
	}

	fn load_panels(&self) -> rusqlite::Result<Vec<Panel>> {
		let mut panels : Vec<Panel> = Vec::new();
		let mut statement = self.conn.prepare("SELECT * FROM panels")?;
		let panels_iter = statement.query_map([], |row| {
			Ok(Panel{
				id: row.get(0)?,
				name: row.get(1)?,
				view_scroll: row.get(2)?,
				view_size: row.get(3)?,
				width: row.get(4)?,
				height: row.get(5)?,
				sources: Mutex::new(Vec::new()),
			})
		})?;

		for panel in panels_iter {
			if let Ok(mut p) = panel {
				p.sources = Mutex::new(self.load_sources(p.id)?);
				panels.push(p);
			}
		}

		Ok(panels)
	}

	fn put_panel(&self, name:&str, view_scroll:bool, view_size:i32, width:i32, height:i32) -> rusqlite::Result<usize> {
		self.conn.execute(
			"INSERT INTO panels (name, view_scroll, view_size, width, height) VALUES (?, ?, ?, ?, ?)",
			params![name, view_scroll, view_size, width, height]
		)
	}

	// jank! TODO make it not jank!
	fn new_panel(&self, name:&str) -> rusqlite::Result<Panel> {
		self.put_panel(name, true, 100, 400, 280)?;
		let mut statement = self.conn.prepare("SELECT * FROM panels WHERE name = ?")?;
		for panel in statement.query_map(params![name], |row| {
			Ok(Panel{
				id: row.get(0)?,
				name: row.get(1)?,
				view_scroll: row.get(2)?,
				view_size: row.get(3)?,
				width: row.get(4)?,
				height: row.get(5)?,
				sources: Mutex::new(Vec::new()),
			})
		})? {
			if let Ok(p) = panel {
				return Ok(p);
			} else {
				println!("WTF");
			}
		}

		Err(rusqlite::Error::QueryReturnedNoRows)
	}

	pub async fn fetch_all(&self) -> Result<(), FetchError> {
		let panels = &*self.panels.lock().unwrap();
		for i in 0..panels.len() {
			let sources = &*panels[i].sources.lock().unwrap();
			for j in 0..sources.len() {
				if !sources[j].valid() {
					let v = sources[j].fetch().await?;
					self.put_value(panels[i].id, sources[j].id, v)?;
					sources[j].data.lock().unwrap().push(v);
				}
			}
		}

		Ok(())
	}

}

impl DataStorage for SQLiteDataStore {
	fn add_panel(&self, name:&str) {
		let panel = self.new_panel(name).unwrap();
		self.panels.lock().unwrap().push(panel);
	}
}