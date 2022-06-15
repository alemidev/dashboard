use super::FetchError;
use chrono::{DateTime, Utc};
use eframe::egui::plot::{Value, Values};
use eframe::epaint::Color32;
use std::sync::RwLock;

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

impl Default for Panel {
	fn default() -> Self {
		Panel {
			id: -1,
			name: "".to_string(),
			view_scroll: true,
			view_size: 300,
			timeserie: true,
			width: 100,
			height: 200,
			limit: false,
		}
	}
}

pub struct Source {
	pub(crate) id: i32,
	pub name: String,
	pub enabled: bool,
	pub url: String,
	pub interval: i32,
	pub color: Color32,
	pub(crate) last_fetch: RwLock<DateTime<Utc>>,
	pub query_x: String,
	// pub(crate) compiled_query_x: Arc<Mutex<jq_rs::JqProgram>>,
	pub query_y: String,
	// pub(crate) compiled_query_y: Arc<Mutex<jq_rs::JqProgram>>,
	pub(crate) panel_id: i32,
	pub(crate) data: RwLock<Vec<Value>>,
}

impl Default for Source {
	fn default() -> Self {
		Source {
			id: -1,
			name: "".to_string(),
			enabled: false,
			url: "".to_string(),
			interval: 60,
			color: Color32::TRANSPARENT,
			last_fetch: RwLock::new(Utc::now()),
			query_x: "".to_string(),
			query_y: "".to_string(),
			panel_id: -1,
			data: RwLock::new(Vec::new()),
		}
	}
}

impl Source {
	pub fn valid(&self) -> bool {
		let last_fetch = self.last_fetch.read().expect("LastFetch RwLock poisoned");
		return (Utc::now() - *last_fetch).num_seconds() < self.interval as i64;
	}

	pub fn values(&self) -> Values {
		Values::from_values(self.data.read().expect("Values RwLock poisoned").clone())
	}

	pub fn values_filter(&self, min_x: f64) -> Values {
		let mut values = self.data.read().expect("Values RwLock poisoned").clone();
		values.retain(|x| x.x > min_x);
		Values::from_values(values)
	}
}

pub fn fetch(url: &str, query_x: &str, query_y: &str) -> Result<Value, FetchError> {
	let res = ureq::get(url).call()?.into_json()?;
	let x: f64;
	if query_x.len() > 0 {
		x = jql::walker(&res, query_x)?
			.as_f64()
			.ok_or(FetchError::JQLError("X query is null".to_string()))?; // TODO what if it's given to us as a string?
	} else {
		x = Utc::now().timestamp() as f64;
	}
	let y = jql::walker(&res, query_y)?
		.as_f64()
		.ok_or(FetchError::JQLError("Y query is null".to_string()))?;
	return Ok(Value { x, y });
}
