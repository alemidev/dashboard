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
	pub diagnostics: RwLock<Vec<String>>,
}

impl ApplicationState {
	pub fn new(path:PathBuf) -> Result<ApplicationState, FetchError> {
		let storage = SQLiteDataStore::new(path.clone())?;

		let panels = storage.load_panels()?;
		let sources = storage.load_sources()?;

		return Ok(ApplicationState{
			run: true,
			file_size: RwLock::new(std::fs::metadata(path.clone())?.len()),
			file_path: path,
			panels: RwLock::new(panels),
			sources: RwLock::new(sources),
			storage: Mutex::new(storage),
			diagnostics: RwLock::new(Vec::new()),
		});
	}

	pub fn add_panel(&self, panel: &Panel) -> Result<(), FetchError> {
		let verified_panel = self.storage.lock().expect("Storage Mutex poisoned")
			.new_panel(
				panel.name.as_str(),
				panel.view_size,
				panel.width,
				panel.height,
				self.panels.read().expect("Panels RwLock poisoned").len() as i32 // todo can this be made more compact and without acquisition?
			)?; // TODO make values customizable and useful
		self.panels.write().expect("Panels RwLock poisoned").push(verified_panel);
		Ok(())
	}

	pub fn add_source(&self, source: &Source) -> Result<(), FetchError> {
		let verified_source = self.storage.lock().expect("Storage Mutex poisoned")
			.new_source(source.panel_id, source.name.as_str(), source.url.as_str(), source.query_x.as_str(), source.query_y.as_str(), source.color, source.visible)?;
		self.sources.write().expect("Sources RwLock poisoned").push(verified_source);
		return Ok(());
	}
}
