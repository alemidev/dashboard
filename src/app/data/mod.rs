pub mod source;
pub mod store;

use std::sync::{Arc, Mutex};
use std::num::ParseFloatError;
use chrono::{DateTime, Utc};
use eframe::egui::plot::{Values, Value};

#[derive(Debug)]
pub enum FetchError {
	ReqwestError(reqwest::Error),
	JqError(jq_rs::Error),
	RusqliteError(rusqlite::Error),
	ParseFloatError(ParseFloatError),
}

impl From::<reqwest::Error> for FetchError {
	fn from(e: reqwest::Error) -> Self { FetchError::ReqwestError(e) }
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



pub struct Panel {
	pub(crate) id: i32,
	pub name: String,
	pub view_scroll: bool,
	pub view_size: i32,
	pub(crate) width: i32,
	pub(crate) height: i32,
	pub(crate) sources: Mutex<Vec<Source>>,
}

impl Panel {
}

pub struct Source {
	pub(crate) id: i32,
	pub name: String,
	pub url: String,
	pub interval: i32,
	pub(crate) last_fetch: DateTime<Utc>,
	pub query_x: String,
	// pub(crate) compiled_query_x: Arc<Mutex<jq_rs::JqProgram>>,
	pub query_y: String,
	// pub(crate) compiled_query_y: Arc<Mutex<jq_rs::JqProgram>>,
	pub(crate) panel_id: i32,
	pub(crate) data: Mutex<Vec<Value>>,
}

impl Source {
	pub fn valid(&self) -> bool {
		return (Utc::now() - self.last_fetch).num_seconds() < self.interval as i64;
	}

	pub fn values(&self) -> Values {
		Values::from_values(self.data.lock().unwrap().clone())
	}

	pub async fn fetch(&self) -> Result<Value, FetchError> {
		let res = reqwest::get(&self.url).await?;
		let body = res.text().await?;
		let x = jq_rs::compile(&self.query_x)?.run(&body)?.parse::<f64>()?;
		let y = jq_rs::compile(&self.query_y)?.run(&body)?.parse::<f64>()?;
		return Ok( Value { x, y } );
	}
}