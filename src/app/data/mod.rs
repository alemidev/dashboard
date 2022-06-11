// pub mod source;
pub mod store;

use std::path::PathBuf;
use std::sync::{RwLock, Mutex};
use std::num::ParseFloatError;
use chrono::{DateTime, Utc};
use eframe::egui::plot::{Values, Value};
use eframe::epaint::Color32;

use self::store::SQLiteDataStore;

#[derive(Debug)]
pub enum FetchError {
	UreqError(ureq::Error),
	IoError(std::io::Error),
	// JqError(jq_rs::Error),
	JQLError(String),
	RusqliteError(rusqlite::Error),
	ParseFloatError(ParseFloatError),
}

impl From::<ureq::Error> for FetchError {
	fn from(e: ureq::Error) -> Self { FetchError::UreqError(e) }
}
impl From::<std::io::Error> for FetchError {
	fn from(e: std::io::Error) -> Self { FetchError::IoError(e) }
}
impl From::<String> for FetchError { // TODO wtf? why does JQL error as a String?
	fn from(e: String) -> Self { FetchError::JQLError(e) }
}
impl From::<ParseFloatError> for FetchError {
	fn from(e: ParseFloatError) -> Self { FetchError::ParseFloatError(e) }
}
impl From::<rusqlite::Error> for FetchError {
	fn from(e: rusqlite::Error) -> Self { FetchError::RusqliteError(e) }
}

pub struct ApplicationState {
	pub run: bool,
	pub file_path: PathBuf,
	pub file_size: RwLock<u64>,
	pub panels: RwLock<Vec<Panel>>,
	pub sources: RwLock<Vec<Source>>,
	pub storage: Mutex<SQLiteDataStore>,
}

impl ApplicationState {
	pub fn new(path:PathBuf) -> Self {
		let storage = SQLiteDataStore::new(path.clone()).unwrap();

		let panels = storage.load_panels().unwrap();
		let sources = storage.load_sources().unwrap();

		return ApplicationState{
			run: true,
			file_size: RwLock::new(std::fs::metadata(path.clone()).unwrap().len()),
			file_path: path,
			panels: RwLock::new(panels),
			sources: RwLock::new(sources),
			storage: Mutex::new(storage),
		};
	}

	pub fn add_panel(&self, name:&str) -> Result<(), FetchError> {
		let panel = self.storage.lock().expect("Storage Mutex poisoned").new_panel(name, 100, 200, 280)?; // TODO make values customizable and useful
		self.panels.write().expect("Panels RwLock poisoned").push(panel);
		Ok(())
	}

	pub fn add_source(&self, panel_id:i32, name:&str, url:&str, query_x:&str, query_y:&str, color:Color32, visible:bool) -> Result<(), FetchError> {
		let source = self.storage.lock().expect("Storage Mutex poisoned")
			.new_source(panel_id, name, url, query_x, query_y, color, visible)?;
		self.sources.write().expect("Sources RwLock poisoned").push(source);
		return Ok(());
	}
}

pub struct Panel {
	pub(crate) id: i32,
	pub name: String,
	pub view_scroll: bool,
	pub view_size: i32,
	pub timeserie: bool,
	pub(crate) width: i32,
	pub(crate) height: i32,
	pub limit: bool,
}

impl Panel {
}

pub struct Source {
	pub(crate) id: i32,
	pub name: String,
	pub url: String,
	pub interval: i32,
	pub color: Color32,
	pub visible: bool,
	pub(crate) last_fetch: RwLock<DateTime<Utc>>,
	pub query_x: String,
	// pub(crate) compiled_query_x: Arc<Mutex<jq_rs::JqProgram>>,
	pub query_y: String,
	// pub(crate) compiled_query_y: Arc<Mutex<jq_rs::JqProgram>>,
	pub(crate) panel_id: i32,
	pub(crate) data: RwLock<Vec<Value>>,
}

impl Source {
	pub fn valid(&self) -> bool {
		let last_fetch = self.last_fetch.read().expect("LastFetch RwLock poisoned");
		return (Utc::now() - *last_fetch).num_seconds() < self.interval as i64;
	}

	pub fn values(&self) -> Values {
		Values::from_values(self.data.read().expect("Values RwLock poisoned").clone())
	}

	pub fn values_filter(&self, min_x:f64) -> Values {
		let mut values = self.data.read().expect("Values RwLock poisoned").clone();
		values.retain(|x| x.x > min_x);
		Values::from_values(values)
	}
}

pub fn fetch(url:&str, query_x:&str, query_y:&str) -> Result<Value, FetchError> {
	let res = ureq::get(url).call()?.into_json()?;
	let x : f64;
	if query_x.len() > 0 {
		x = jql::walker(&res, query_x)?.as_f64().unwrap(); // TODO what if it's given to us as a string?
	} else {
		x = Utc::now().timestamp() as f64;
	}
	let y = jql::walker(&res, query_y)?.as_f64().unwrap();
	return Ok( Value { x, y } );
}