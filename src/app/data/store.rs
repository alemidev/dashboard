use crate::app::util::unpack_color;
use crate::app::{
	data::source::{Panel, Source},
	util::repack_color,
};
use chrono::{TimeZone, Utc};
use eframe::egui::{plot::Value, Color32};
use rusqlite::{params, Connection};
use std::sync::RwLock;

use super::source::Metric;

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
				position INT NOT NULL,
				reduce_view BOOL NOT NULL,
				view_chunks INT NOT NULL,
				shift_view BOOL NOT NULL,
				view_offset INT NOT NULL
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
				position INT NOT NULL
			);",
			[],
		)?;

		conn.execute(
			"CREATE TABLE IF NOT EXISTS metrics (
				id INTEGER PRIMARY KEY,
				name TEXT NOT NULL,
				source_id INT NOT NULL,
				query_x TEXT NOT NULL,
				query_y TEXT NOT NULL,
				panel_id INT NOT NULL,
				color INT NOT NULL,
				position INT NOT NULL
			);",
			[],
		)?;

// BEGIN TRANSACTION;
// CREATE TEMPORARY TABLE t1_backup(a,b);
// INSERT INTO t1_backup SELECT a,b FROM t1;
// DROP TABLE t1;
// CREATE TABLE t1(a,b);
// INSERT INTO t1 SELECT a,b FROM t1_backup;
// DROP TABLE t1_backup;
// COMMIT;

		conn.execute(
			"CREATE TABLE IF NOT EXISTS points (
				id INTEGER PRIMARY KEY,
				metric_id INT NOT NULL,
				x FLOAT NOT NULL,
				y FLOAT NOT NULL
			);",
			[],
		)?;

		Ok(SQLiteDataStore { conn })
	}

	pub fn load_values(&self, metric_id: i32) -> rusqlite::Result<Vec<Value>> {
		let mut values: Vec<Value> = Vec::new();
		let mut statement = self
			.conn
			.prepare("SELECT x, y FROM points WHERE metric_id = ?")?;
		let values_iter = statement.query_map(params![metric_id], |row| {
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

	pub fn put_value(&self, metric_id: i32, v: Value) -> rusqlite::Result<usize> {
		self.conn.execute(
			"INSERT INTO points(metric_id, x, y) VALUES (?, ?, ?)",
			params![metric_id, v.x, v.y],
		)
	}

	pub fn delete_values(&self, metric_id: i32) -> rusqlite::Result<usize> {
		self.conn.execute(
			"DELETE FROM points WHERE metric_id = ?",
			params![metric_id]
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
			})
		})?;

		for source in sources_iter {
			if let Ok(s) = source {
				sources.push(s);
			}
		}

		Ok(sources)
	}

	// jank! TODO make it not jank!
	pub fn new_source(
		&self,
		name: &str,
		enabled: bool,
		url: &str,
		interval: i32,
		position: i32,
	) -> rusqlite::Result<Source> {
		self.conn.execute(
			"INSERT INTO sources(name, enabled, url, interval, position) VALUES (?, ?, ?, ?, ?)",
			params![name, enabled, url, interval, position],
		)?;
		let mut statement = self
			.conn
			.prepare("SELECT * FROM sources WHERE name = ? AND url = ? ORDER BY id DESC")?;
		for panel in statement.query_map(params![name, url], |row| {
			Ok(Source {
				id: row.get(0)?,
				name: row.get(1)?,
				enabled: row.get(2)?,
				url: row.get(3)?,
				interval: row.get(4)?,
				// position: row.get(5)?,
				last_fetch: RwLock::new(Utc.ymd(1970, 1, 1).and_hms(0, 0, 0)),
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
		id: i32,
		name: &str,
		enabled: bool,
		url: &str,
		interval: i32,
		position: i32,
	) -> rusqlite::Result<usize> {
		self.conn.execute(
			"UPDATE sources SET name = ?, enabled = ?, url = ?, interval = ?, position = ? WHERE id = ?",
			params![name, enabled, url, interval, position, id],
		)
	}

	pub fn delete_source(&self, id:i32) -> rusqlite::Result<usize> {
		self.conn.execute("DELETE FROM sources WHERE id = ?", params![id])
	}

	pub fn load_metrics(&self) -> rusqlite::Result<Vec<Metric>> {
		let mut metrics: Vec<Metric> = Vec::new();
		let mut statement = self.conn.prepare("SELECT * FROM metrics ORDER BY position")?;
		let metrics_iter = statement.query_map([], |row| {
			Ok(Metric {
				id: row.get(0)?,
				name: row.get(1)?,
				source_id: row.get(2)?,
				query_x: row.get(3)?,
				query_y: row.get(4)?,
				panel_id: row.get(5)?,
				color: unpack_color(row.get(6).unwrap_or(0)),
				// position: row.get(7)?,
				data: RwLock::new(Vec::new()),
			})
		})?;

		for metric in metrics_iter {
			if let Ok(m) = metric {
				*m.data.write().expect("Points RwLock poisoned") = self.load_values(m.id)?;
				metrics.push(m);
			}
		}

		Ok(metrics)
	}

	// jank! TODO make it not jank!
	pub fn new_metric(
		&self,
		name: &str,
		source_id: i32,
		query_x: &str,
		query_y: &str,
		panel_id: i32,
		color: Color32,
		position: i32,
	) -> rusqlite::Result<Metric> {
		self.conn.execute(
			"INSERT INTO metrics(name, source_id, query_x, query_y, panel_id, color, position) VALUES (?, ?, ?, ?, ?, ?, ?)",
			params![name, source_id, query_x, query_y, panel_id, repack_color(color), position],
		)?;
		let mut statement = self
			.conn
			.prepare("SELECT * FROM metrics WHERE source_id = ? AND panel_id = ? AND name = ? ORDER BY id DESC")?;
		for metric in statement.query_map(params![source_id, panel_id, name], |row| {
			Ok(Metric {
				id: row.get(0)?,
				name: row.get(1)?,
				source_id: row.get(2)?,
				query_x: row.get(3)?,
				query_y: row.get(4)?,
				panel_id: row.get(5)?,
				color: unpack_color(row.get(6).unwrap_or(0)),
				// position: row.get(7)?,
				data: RwLock::new(Vec::new()),
			})
		})? {
			if let Ok(m) = metric {
				return Ok(m);
			}
		}

		Err(rusqlite::Error::QueryReturnedNoRows)
	}

	pub fn update_metric(
		&self,
		id: i32,
		name: &str,
		source_id: i32,
		query_x: &str,
		query_y: &str,
		panel_id: i32,
		color: Color32,
		position: i32,
	) -> rusqlite::Result<usize> {
		self.conn.execute(
			"UPDATE metrics SET name = ?, query_x = ?, query_y = ?, panel_id = ?, color = ?, position = ? WHERE id = ? AND source_id = ?",
			params![name, query_x, query_y, panel_id, repack_color(color), position, id, source_id],
		)
	}

	pub fn delete_metric(&self, id:i32) -> rusqlite::Result<usize> {
		self.conn.execute("DELETE FROM metrics WHERE id = ?", params![id])
	}

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
				// position: row.get(8)?,
				reduce: row.get(9)?,
				view_chunks: row.get(10)?,
				shift: row.get(11)?,
				view_offset: row.get(12)?,
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
		view_scroll: bool,
		view_size: u32,
		view_chunks: u32,
		view_offset: u32,
		timeserie: bool,
		width: i32,
		height: i32,
		limit: bool,
		reduce: bool,
		shift: bool,
		position: i32,
	) -> rusqlite::Result<Panel> {
		self.conn.execute(
			"INSERT INTO panels (name, view_scroll, view_size, timeserie, width, height, limit_view, position, reduce_view, view_chunks, shift_view, view_offset) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
			params![name, view_scroll, view_size, timeserie, width, height, limit, position, reduce, view_chunks, shift, view_offset]
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
				// position: row.get(8)?,
				reduce: row.get(9)?,
				view_chunks: row.get(10)?,
				shift: row.get(11)?,
				view_offset: row.get(12)?,
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
		view_size: u32,
		view_chunks: u32,
		view_offset: u32,
		timeserie: bool,
		width: i32,
		height: i32,
		limit: bool,
		reduce: bool,
		shift: bool,
		position: i32,
	) -> rusqlite::Result<usize> {
		self.conn.execute(
			"UPDATE panels SET name = ?, view_scroll = ?, view_size = ?, timeserie = ?, width = ?, height = ?, limit_view = ?, position = ?, reduce_view = ?, view_chunks = ?, shift_view = ?, view_offset = ? WHERE id = ?",
			params![name, view_scroll, view_size, timeserie, width, height, limit, position, reduce, view_chunks, shift, view_offset, id],
		)
	}

	pub fn delete_panel(&self, id:i32) -> rusqlite::Result<usize> {
		self.conn.execute("DELETE FROM panels WHERE id = ?", params![id])
	}
}
