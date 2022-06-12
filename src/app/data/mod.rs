pub mod source;
pub mod store;

use std::path::PathBuf;
use std::sync::{RwLock, Mutex};
use std::num::ParseFloatError;
use eframe::epaint::Color32;
use self::store::SQLiteDataStore;
use self::source::{Panel, Source};

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
