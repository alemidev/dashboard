// pub mod source;
pub mod store;

use std::path::PathBuf;
use std::sync::{RwLock, Mutex};
use std::num::ParseFloatError;
use chrono::{DateTime, Utc};
use eframe::egui::plot::{Values, Value};

use self::store::SQLiteDataStore;

#[derive(Debug)]
pub enum FetchError {
	UreqError(ureq::Error),
	IoError(std::io::Error),
	JqError(jq_rs::Error),
	RusqliteError(rusqlite::Error),
	ParseFloatError(ParseFloatError),
	NoPanelWithThatIdError,
}

impl From::<ureq::Error> for FetchError {
	fn from(e: ureq::Error) -> Self { FetchError::UreqError(e) }
}
impl From::<std::io::Error> for FetchError {
	fn from(e: std::io::Error) -> Self { FetchError::IoError(e) }
}
impl From::<jq_rs::Error> for FetchError {
	fn from(e: jq_rs::Error) -> Self { FetchError::JqError(e) }
}
impl From::<ParseFloatError> for FetchError {
	fn from(e: ParseFloatError) -> Self { FetchError::ParseFloatError(e) }
}
impl From::<rusqlite::Error> for FetchError {
	fn from(e: rusqlite::Error) -> Self { FetchError::RusqliteError(e) }
}

pub struct ApplicationState {
	pub run: bool,
	pub panels: RwLock<Vec<Panel>>,
	pub storage: Mutex<SQLiteDataStore>,
}

impl ApplicationState {
	pub fn new(path:PathBuf) -> Self {
		let storage = SQLiteDataStore::new(path).unwrap();

		let panels = storage.load_panels().unwrap();

		return ApplicationState{
			run: true,
			panels: RwLock::new(panels),
			storage: Mutex::new(storage),
		};
	}

	pub fn add_panel(&self, name:&str) -> Result<(), FetchError> {
		let panel = self.storage.lock().unwrap().new_panel(name, 100, 200, 280)?; // TODO make values customizable and useful
		self.panels.write().unwrap().push(panel);
		Ok(())
	}

	pub fn add_source(&self, panel_id:i32, name:&str, url:&str, query_x:&str, query_y:&str) -> Result<(), FetchError> {
		let source = self.storage.lock().unwrap().new_source(panel_id, name, url, query_x, query_y)?;
		let panels = self.panels.read().unwrap();
		for panel in &*panels {
			if panel.id == panel_id {
				panel.sources.write().unwrap().push(source);
				return Ok(());
			}
		}
		Err(FetchError::NoPanelWithThatIdError)
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
	pub(crate) sources: RwLock<Vec<Source>>,

}

impl Panel {
}

pub struct Source {
	pub(crate) id: i32,
	pub name: String,
	pub url: String,
	pub interval: i32,
	pub(crate) last_fetch: RwLock<DateTime<Utc>>,
	pub query_x: String,
	// pub(crate) compiled_query_x: Arc<Mutex<jq_rs::JqProgram>>,
	pub query_y: String,
	// pub(crate) compiled_query_y: Arc<Mutex<jq_rs::JqProgram>>,
	// pub(crate) panel_id: i32,
	pub(crate) data: RwLock<Vec<Value>>,
}

impl Source {
	pub fn valid(&self) -> bool {
		let last_fetch = self.last_fetch.read().unwrap();
		return (Utc::now() - *last_fetch).num_seconds() < self.interval as i64;
	}

	pub fn values(&self) -> Values {
		Values::from_values(self.data.read().unwrap().clone())
	}

	pub fn values_filter(&self, min_x:f64) -> Values {
		let mut values = self.data.read().unwrap().clone();
		values.retain(|x| x.x > min_x);
		Values::from_values(values)
	}

	// Not really useful since different data has different fetch rates
	// pub fn values_limit(&self, size:usize) -> Values {
	// 	let values = self.data.read().unwrap().clone(); 
	// 	let min = if values.len() < size { 0 } else { values.len() - size };
	// 	Values::from_values(values[min..values.len()].to_vec())
	// }
}

pub fn fetch(url:&str, query_x:&str, query_y:&str) -> Result<Value, FetchError> {
	let res = ureq::get(url).call()?;
	let body = res.into_string()?;
	let x : f64;
	if query_x.len() > 0 {
		x = jq_rs::compile(query_x)?.run(&body)?.trim().parse::<f64>()?; // TODO precompile and guard with a mutex
	} else {
		x = Utc::now().timestamp() as f64;
	}
	let y = jq_rs::compile(query_y)?.run(&body)?.trim().parse::<f64>()?;
	return Ok( Value { x, y } );
}