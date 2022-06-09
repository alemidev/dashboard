use crate::app::{data::{Panel, Source}, util::repack_color};
use chrono::{TimeZone, Utc};
use eframe::egui::{Color32, plot::Value};
use rusqlite::{params, Connection};
use std::sync::RwLock;
use crate::app::util::unpack_color;

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
				name TEXT UNIQUE NOT NULL,
				view_scroll BOOL NOT NULL,
				view_size INT NOT NULL,
				timeserie BOOL NOT NULL,
				width INT NOT NULL,
				height INT NOT NULL
			);",
			[],
		)?;

		conn.execute(
			"CREATE TABLE IF NOT EXISTS sources (
				id INTEGER PRIMARY KEY,
				name TEXT NOT NULL,
				url TEXT NOT NULL,
				interval INT NOT NULL,
				query_x TEXT NOT NULL,
				query_y TEXT NOT NULL,
				panel_id INT NOT NULL,
				color INT NULL,
				visible BOOL NOT NULL
			);",
			[],
		)?;

		conn.execute(
			"CREATE TABLE IF NOT EXISTS points (
				id INTEGER PRIMARY KEY,
				source_id INT NOT NULL,
				x FLOAT NOT NULL,
				y FLOAT NOT NULL
			);",
			[],
		)?;

		Ok(SQLiteDataStore { conn })
	}

	

	pub fn load_values(&self, source_id: i32) -> rusqlite::Result<Vec<Value>> {
		let mut values: Vec<Value> = Vec::new();
		let mut statement = self
			.conn
			.prepare("SELECT x, y FROM points WHERE source_id = ?")?;
		let values_iter = statement.query_map(params![source_id], |row| {
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

	pub fn put_value(&self, source_id: i32, v: Value) -> rusqlite::Result<usize> {
		self.conn.execute(
			"INSERT INTO points(source_id, x, y) VALUES (?, ?, ?)",
			params![source_id, v.x, v.y],
		)
	}



	pub fn load_sources(&self) -> rusqlite::Result<Vec<Source>> {
		let mut sources: Vec<Source> = Vec::new();
		let mut statement = self
			.conn
			.prepare("SELECT * FROM sources")?;
		let sources_iter = statement.query_map([], |row| {
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
				color: unpack_color(row.get(7).unwrap_or(0)),
				visible: row.get(8)?,
				data: RwLock::new(Vec::new()),
			})
		})?;

		for source in sources_iter {
			if let Ok(mut s) = source {
				s.data = RwLock::new(self.load_values(s.id)?);
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
		color: Color32,
		visible: bool,
	) -> rusqlite::Result<Source> {
		let color_u32 : Option<u32> = if color == Color32::TRANSPARENT { None } else { Some(repack_color(color)) };
		self.conn.execute(
			"INSERT INTO sources(name, url, interval, query_x, query_y, panel_id, color, visible) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
			params![name, url, 60i32, query_x, query_y, panel_id, color_u32, visible],
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
				last_fetch: RwLock::new(Utc.ymd(1970, 1, 1).and_hms(0, 0, 0)),
				query_x: row.get(4)?,
				// compiled_query_x: Arc::new(Mutex::new(jq_rs::compile(row.get::<usize, String>(4)?.as_str()).unwrap())),
				query_y: row.get(5)?,
				// compiled_query_y: Arc::new(Mutex::new(jq_rs::compile(row.get::<usize, String>(5)?.as_str()).unwrap())),
				panel_id: row.get(6)?,
				color: unpack_color(row.get(7).unwrap_or(0)),
				visible: row.get(8)?,
				data: RwLock::new(Vec::new()),
			})
		})? {
			if let Ok(p) = panel {
				return Ok(p);
			}
		}

		Err(rusqlite::Error::QueryReturnedNoRows)
	}

	pub fn update_source(
		&self,
		source_id: i32,
		panel_id: i32,
		name: &str,
		url: &str,
		interval: i32,
		query_x: &str,
		query_y: &str,
		color: Color32,
		visible: bool,
	) -> rusqlite::Result<usize> {
		let color_u32 : Option<u32> = if color == Color32::TRANSPARENT { None } else { Some(repack_color(color)) };
		self.conn.execute(
			"UPDATE sources SET name = ?, url = ?, interval = ?, query_x = ?, query_y = ?, panel_id = ?, color = ?, visible = ? WHERE id = ?",
			params![name, url, interval, query_x, query_y, panel_id, color_u32, visible, source_id],
		)
	}

	// pub fn delete_source(&self, id:i32) -> rusqlite::Result<usize> {
	// 	self.conn.execute("DELETE FROM sources WHERE id = ?", params![id])
	// }

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
			})
		})?;

		for panel in panels_iter {
			if let Ok(p) = panel {
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

	// pub fn delete_panel(&self, id:i32) -> rusqlite::Result<usize> {
	// 	self.conn.execute("DELETE FROM panels WHERE id = ?", params![id])
	// }



}
