use crate::app::data::{Panel, Source};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use eframe::egui::plot::Value;
use rusqlite::{params, Connection};
use std::sync::{Arc, RwLock};

use super::FetchError;

pub trait DataStorage {
	fn add_panel(&self, name: &str);
}

pub struct SQLiteDataStore {
	conn: Connection,
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
				timeserie BOOL,
				width INT,
				height INT
			);",
			[],
		)?;

		conn.execute(
			"CREATE TABLE IF NOT EXISTS sources (
				id INTEGER PRIMARY KEY,
				name TEXT,
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

		Ok(SQLiteDataStore { conn })
	}

	

	pub fn load_values(&self, panel_id: i32, source_id: i32) -> rusqlite::Result<Vec<Value>> {
		let mut values: Vec<Value> = Vec::new();
		let mut statement = self
			.conn
			.prepare("SELECT x, y FROM points WHERE panel_id = ? AND source_id = ?")?;
		let values_iter = statement.query_map(params![panel_id, source_id], |row| {
			Ok(Value {
				x: row.get(0)?,
				y: row.get(1)?,
			})
		})?;

		for value in values_iter {
			if let Ok(v) = value {
				values.push(v);
			}
		}

		Ok(values)
	}

	pub fn put_value(&self, panel_id: i32, source_id: i32, v: Value) -> rusqlite::Result<usize> {
		self.conn.execute(
			"INSERT INTO points(panel_id, source_id, x, y) VALUES (?, ?, ?, ?)",
			params![panel_id, source_id, v.x, v.y],
		)
	}



	pub fn load_sources(&self, panel_id: i32) -> rusqlite::Result<Vec<Source>> {
		let mut sources: Vec<Source> = Vec::new();
		let mut statement = self
			.conn
			.prepare("SELECT * FROM sources WHERE panel_id = ?")?;
		let sources_iter = statement.query_map(params![panel_id], |row| {
			Ok(Source {
				id: row.get(0)?,
				name: row.get(1)?,
				url: row.get(2)?,
				interval: row.get(3)?,
				last_fetch: RwLock::new(Utc.ymd(1970, 1, 1).and_hms(0, 0, 0)),
				query_x: row.get(4)?,
				// compiled_query_x: Arc::new(Mutex::new(jq_rs::compile(row.get::<usize, String>(4)?.as_str()).unwrap())),
				query_y: row.get(5)?,
				// compiled_query_y: Arc::new(Mutex::new(jq_rs::compile(row.get::<usize, String>(5)?.as_str()).unwrap())),
				panel_id: row.get(6)?,
				data: RwLock::new(Vec::new()),
			})
		})?;

		for source in sources_iter {
			if let Ok(mut s) = source {
				s.data = RwLock::new(self.load_values(panel_id, s.id)?);
				sources.push(s);
			}
		}

		Ok(sources)
	}

	// jank! TODO make it not jank!
	pub fn new_source(
		&self,
		panel_id: i32,
		name: &str,
		url: &str,
		query_x: &str,
		query_y: &str,
	) -> rusqlite::Result<Source> {
		self.conn.execute(
			"INSERT INTO sources(name, url, interval, query_x, query_y, panel_id) VALUES (?, ?, ?, ?, ?, ?)",
			params![name, url, 60, query_x, query_y, panel_id],
		)?;
		let mut statement = self
			.conn
			.prepare("SELECT * FROM sources WHERE name = ? AND panel_id = ?")?;
		for panel in statement.query_map(params![name, panel_id], |row| {
			Ok(Source {
				id: row.get(0)?,
				name: row.get(1)?,
				url: row.get(2)?,
				interval: row.get(3)?,
				query_x: row.get(4)?,
				query_y: row.get(5)?,
				panel_id: row.get(6)?,
				last_fetch: RwLock::new(Utc.ymd(1970, 1, 1).and_hms(0, 0, 0)),
				data: RwLock::new(Vec::new()),
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

	pub fn update_source(
		&self,
		source_id: i32,
		name: &str,
		url: &str,
		interval: i32,
		query_x: &str,
		query_y: &str,
	) -> rusqlite::Result<usize> {
		self.conn.execute(
			"UPDATE sources SET name = ?, url = ?, interval = ?, query_x = ?, query_y = ? WHERE id = ?",
			params![name, url, interval, query_x, query_y, source_id],
		)
	}

	pub fn delete_source(&self, id:i32) -> rusqlite::Result<usize> {
		self.conn.execute("DELETE FROM sources WHERE id = ?", params![id])
	}



	pub fn load_panels(&self) -> rusqlite::Result<Vec<Panel>> {
		let mut panels: Vec<Panel> = Vec::new();
		let mut statement = self.conn.prepare("SELECT * FROM panels")?;
		let panels_iter = statement.query_map([], |row| {
			Ok(Panel {
				id: row.get(0)?,
				name: row.get(1)?,
				view_scroll: row.get(2)?,
				view_size: row.get(3)?,
				timeserie: row.get(4)?,
				width: row.get(5)?,
				height: row.get(6)?,
				sources: RwLock::new(Vec::new()),
			})
		})?;

		for panel in panels_iter {
			if let Ok(mut p) = panel {
				p.sources = RwLock::new(self.load_sources(p.id)?);
				panels.push(p);
			}
		}

		Ok(panels)
	}

	// jank! TODO make it not jank!
	pub fn new_panel(&self, name: &str, view_size:i32, width: i32, height: i32) -> rusqlite::Result<Panel> {
		self.conn.execute(
			"INSERT INTO panels (name, view_scroll, view_size, timeserie, width, height) VALUES (?, ?, ?, ?, ?, ?)",
			params![name, true, view_size, true, width, height]
		)?;
		let mut statement = self.conn.prepare("SELECT * FROM panels WHERE name = ?")?;
		for panel in statement.query_map(params![name], |row| {
			Ok(Panel {
				id: row.get(0)?,
				name: row.get(1)?,
				view_scroll: row.get(2)?,
				view_size: row.get(3)?,
				timeserie: row.get(4)?,
				width: row.get(5)?,
				height: row.get(6)?,
				sources: RwLock::new(Vec::new()),
			})
		})? {
			if let Ok(p) = panel {
				return Ok(p);
			}
		}
		Err(rusqlite::Error::QueryReturnedNoRows)
	}

	pub fn update_panel(
		&self,
		id: i32,
		name: &str,
		view_scroll: bool,
		view_size: i32,
		timeserie: bool,
		width: i32,
		height: i32,
	) -> rusqlite::Result<usize> {
		self.conn.execute(
			"UPDATE panels SET name = ?, view_scroll = ?, view_size = ?, timeserie = ?, width = ?, height = ? WHERE id = ?",
			params![name, view_scroll, view_size, timeserie, width, height, id],
		)
	}

	pub fn delete_panel(&self, id:i32) -> rusqlite::Result<usize> {
		self.conn.execute("DELETE FROM panels WHERE id = ?", params![id])
	}



}
