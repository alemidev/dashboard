use crate::app::util::unpack_color;
use crate::app::{
	data::source::{Panel, Source},
	util::repack_color,
};
use chrono::{TimeZone, Utc};
use eframe::egui::{plot::Value, Color32};
use rusqlite::{params, Connection};
use std::sync::RwLock;

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
				height INT NOT NULL,
				limit_view BOOL NOT NULL,
				position INT NOT NULL
			);",
			[],
		)?;

		conn.execute(
			"CREATE TABLE IF NOT EXISTS sources (
				id INTEGER PRIMARY KEY,
				name TEXT NOT NULL,
				enabled BOOL NOT NULL,
				url TEXT NOT NULL,
				interval INT NOT NULL,
				query_x TEXT NOT NULL,
				query_y TEXT NOT NULL,
				panel_id INT NOT NULL,
				color INT NULL,
				position INT NOT NULL
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
		let mut statement = self.conn.prepare("SELECT * FROM sources ORDER BY position")?;
		let sources_iter = statement.query_map([], |row| {
			Ok(Source {
				id: row.get(0)?,
				name: row.get(1)?,
				enabled: row.get(2)?,
				url: row.get(3)?,
				interval: row.get(4)?,
				last_fetch: RwLock::new(Utc.ymd(1970, 1, 1).and_hms(0, 0, 0)),
				query_x: row.get(5)?,
				query_y: row.get(6)?,
				panel_id: row.get(7)?,
				color: unpack_color(row.get(8).unwrap_or(0)),
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
		enabled: bool,
		url: &str,
		interval: i32,
		query_x: &str,
		query_y: &str,
		color: Color32,
		position: i32,
	) -> rusqlite::Result<Source> {
		let color_u32: Option<u32> = if color == Color32::TRANSPARENT {
			None
		} else {
			Some(repack_color(color))
		};
		self.conn.execute(
			"INSERT INTO sources(name, enabled, url, interval, query_x, query_y, panel_id, color, position) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
			params![name, enabled, url, interval, query_x, query_y, panel_id, color_u32, position],
		)?;
		let mut statement = self
			.conn
			.prepare("SELECT * FROM sources WHERE name = ? AND panel_id = ?")?;
		for panel in statement.query_map(params![name, panel_id], |row| {
			Ok(Source {
				id: row.get(0)?,
				name: row.get(1)?,
				enabled: row.get(2)?,
				url: row.get(3)?,
				interval: row.get(4)?,
				last_fetch: RwLock::new(Utc.ymd(1970, 1, 1).and_hms(0, 0, 0)),
				query_x: row.get(5)?,
				query_y: row.get(6)?,
				panel_id: row.get(7)?,
				color: unpack_color(row.get(8).unwrap_or(0)),
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
		enabled: bool,
		url: &str,
		interval: i32,
		query_x: &str,
		query_y: &str,
		color: Color32,
		position: i32,
	) -> rusqlite::Result<usize> {
		let color_u32: Option<u32> = if color == Color32::TRANSPARENT {
			None
		} else {
			Some(repack_color(color))
		};
		self.conn.execute(
			"UPDATE sources SET name = ?, enabled = ?, url = ?, interval = ?, query_x = ?, query_y = ?, panel_id = ?, color = ?, position = ? WHERE id = ?",
			params![name, enabled, url, interval, query_x, query_y, panel_id, color_u32, position, source_id],
		)
	}

	// pub fn delete_source(&self, id:i32) -> rusqlite::Result<usize> {
	// 	self.conn.execute("DELETE FROM sources WHERE id = ?", params![id])
	// }

	pub fn load_panels(&self) -> rusqlite::Result<Vec<Panel>> {
		let mut panels: Vec<Panel> = Vec::new();
		let mut statement = self
			.conn
			.prepare("SELECT * FROM panels ORDER BY position")?;
		let panels_iter = statement.query_map([], |row| {
			Ok(Panel {
				id: row.get(0)?,
				name: row.get(1)?,
				view_scroll: row.get(2)?,
				view_size: row.get(3)?,
				timeserie: row.get(4)?,
				width: row.get(5)?,
				height: row.get(6)?,
				limit: row.get(7)?,
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
	pub fn new_panel(
		&self,
		name: &str,
		view_size: i32,
		width: i32,
		height: i32,
		position: i32,
	) -> rusqlite::Result<Panel> {
		self.conn.execute(
			"INSERT INTO panels (name, view_scroll, view_size, timeserie, width, height, limit_view, position) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
			params![name, true, view_size, true, width, height, false, position]
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
				limit: row.get(7)?,
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
		limit: bool,
		position: i32,
	) -> rusqlite::Result<usize> {
		self.conn.execute(
			"UPDATE panels SET name = ?, view_scroll = ?, view_size = ?, timeserie = ?, width = ?, height = ?, limit_view = ?, position = ? WHERE id = ?",
			params![name, view_scroll, view_size, timeserie, width, height, limit, position, id],
		)
	}

	pub fn delete_panel(&self, id:i32) -> rusqlite::Result<usize> {
		self.conn.execute("DELETE FROM panels WHERE id = ?", params![id])
	}
}
