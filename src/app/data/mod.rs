pub mod source;
pub mod store;

use self::source::{Panel, Source, Metric};
use self::store::SQLiteDataStore;
use std::num::ParseFloatError;
use std::path::PathBuf;
use std::sync::{Mutex, RwLock};

#[derive(Debug)]
pub enum FetchError {
	UreqError(ureq::Error),
	IoError(std::io::Error),
	// JqError(jq_rs::Error),
	JQLError(String),
	RusqliteError(rusqlite::Error),
	ParseFloatError(ParseFloatError),
}

impl From<ureq::Error> for FetchError {
	fn from(e: ureq::Error) -> Self {
		FetchError::UreqError(e)
	}
}
impl From<std::io::Error> for FetchError {
	fn from(e: std::io::Error) -> Self {
		FetchError::IoError(e)
	}
}
impl From<String> for FetchError {
	// TODO wtf? why does JQL error as a String?
	fn from(e: String) -> Self {
		FetchError::JQLError(e)
	}
}
impl From<ParseFloatError> for FetchError {
	fn from(e: ParseFloatError) -> Self {
		FetchError::ParseFloatError(e)
	}
}
impl From<rusqlite::Error> for FetchError {
	fn from(e: rusqlite::Error) -> Self {
		FetchError::RusqliteError(e)
	}
}

pub struct ApplicationState {
	pub run: bool,
	pub file_path: PathBuf,
	pub file_size: RwLock<u64>,
	pub panels: RwLock<Vec<Panel>>,
	pub sources: RwLock<Vec<Source>>,
	pub metrics: RwLock<Vec<Metric>>,
	pub storage: Mutex<SQLiteDataStore>,
	pub diagnostics: RwLock<Vec<String>>,
}

impl ApplicationState {
	pub fn new(path: PathBuf) -> Result<ApplicationState, FetchError> {
		let storage = SQLiteDataStore::new(path.clone())?;

		let panels = storage.load_panels()?;
		let sources = storage.load_sources()?;
		let metrics = storage.load_metrics()?;

		return Ok(ApplicationState {
			run: true,
			file_size: RwLock::new(std::fs::metadata(path.clone())?.len()),
			file_path: path,
			panels: RwLock::new(panels),
			sources: RwLock::new(sources),
			metrics: RwLock::new(metrics),
			storage: Mutex::new(storage),
			diagnostics: RwLock::new(Vec::new()),
		});
	}

	pub fn add_panel(&self, panel: &Panel) -> Result<(), FetchError> {
		let verified_panel = self
			.storage
			.lock()
			.expect("Storage Mutex poisoned")
			.new_panel(
				panel.name.as_str(),
				false,
				panel.view_size,
				5,
				0,
				true,
				panel.width,
				panel.height,
				false,
				false,
				false,
				true,
				self.panels.read().expect("Panels RwLock poisoned").len() as i32, // todo can this be made more compact and without acquisition?
			)?; // TODO make values customizable and useful
		self.panels
			.write()
			.expect("Panels RwLock poisoned")
			.push(verified_panel);
		Ok(())
	}

	pub fn add_source(&self, source: &Source) -> Result<(), FetchError> {
		let verified_source = self
			.storage
			.lock()
			.expect("Storage Mutex poisoned")
			.new_source(
				source.name.as_str(),
				source.enabled,
				source.url.as_str(),
				source.interval,
				self.sources.read().expect("Sources RwLock poisoned").len() as i32,
			)?;
		self.sources
			.write()
			.expect("Sources RwLock poisoned")
			.push(verified_source);
		return Ok(());
	}

	pub fn add_metric(&self, metric: &Metric, source: &Source) -> Result<(), FetchError> {
		let verified_metric = self
			.storage
			.lock()
			.expect("Storage Mutex poisoned")
			.new_metric(
				metric.name.as_str(),
				source.id,
				metric.query_x.as_str(),
				metric.query_y.as_str(),
				metric.panel_id,
				metric.color,
				self.metrics.read().expect("Sources RwLock poisoned").len() as i32, // TODO use source.metrics.len()
			)?;
		self.metrics
			.write()
			.expect("Sources RwLock poisoned")
			.push(verified_metric);
		return Ok(());
	}
}
