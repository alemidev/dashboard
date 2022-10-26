use super::FetchError;
use chrono::{DateTime, Utc};
use eframe::egui::plot::PlotPoint;
use eframe::epaint::Color32;
use std::sync::RwLock;

#[derive(Debug, Clone)]
pub struct Panel {
	pub(crate) id: i32,
	pub name: String,
	pub view_scroll: bool,
	pub view_size: u32,
	pub view_chunks: u32,
	pub view_offset: u32,
	pub timeserie: bool,
	pub(crate) width: i32,
	pub(crate) height: i32,
	pub limit: bool,
	pub reduce: bool,
	pub shift: bool,
	pub average: bool,
}

impl Default for Panel {
	fn default() -> Self {
		Panel {
			id: -1,
			name: "".to_string(),
			view_scroll: true,
			view_size: 300,
			view_chunks: 5,
			view_offset: 0,
			timeserie: true,
			width: 100,
			height: 200,
			limit: false,
			reduce: false,
			shift: false,
			average: false,
		}
	}
}

#[derive(Debug)]
pub struct Source {
	pub(crate) id: i32,
	pub name: String,
	pub enabled: bool,
	pub url: String,
	pub interval: i32,
	pub(crate) last_fetch: RwLock<DateTime<Utc>>,
}

impl Default for Source {
	fn default() -> Self {
		Source {
			id: -1,
			name: "".to_string(),
			enabled: false,
			url: "".to_string(),
			interval: 60,
			last_fetch: RwLock::new(Utc::now()),
		}
	}
}

fn avg_value(values: &[PlotPoint]) -> PlotPoint {
	let mut x = 0.0;
	let mut y = 0.0;
	for v in values {
		x += v.x;
		y += v.y;
	}
	return PlotPoint {
		x: x / values.len() as f64,
		y: y / values.len() as f64,
	};
}

impl Source {
	pub fn valid(&self) -> bool {
		let last_fetch = self.last_fetch.read().expect("LastFetch RwLock poisoned");
		return (Utc::now() - *last_fetch).num_seconds() < self.interval as i64;
	}

	// pub fn fetch(&self) -> Result<serde_json::Value, FetchError> {
	// 	fetch(self.url.as_str())
	// }
}

pub fn fetch(url: &str) -> Result<serde_json::Value, FetchError> {
	return Ok(ureq::get(url).call()?.into_json()?);
}

#[derive(Debug)]
pub struct Metric {
	pub(crate) id: i32,
	pub name: String,
	pub source_id: i32,
	pub color: Color32,
	pub query_x: String,
	pub query_y: String,
	pub(crate) panel_id: i32,
	pub(crate) data: RwLock<Vec<PlotPoint>>,
}

impl Default for Metric {
	fn default() -> Self {
		Metric {
			id: -1,
			name: "".to_string(),
			source_id: -1,
			color: Color32::TRANSPARENT,
			query_x: "".to_string(),
			query_y: "".to_string(),
			panel_id: -1,
			data: RwLock::new(Vec::new()),
		}
	}
}

impl Metric {
	pub fn extract(&self, value: &serde_json::Value) -> Result<PlotPoint, FetchError> {
		let x: f64;
		if self.query_x.len() > 0 {
			x = jql::walker(value, self.query_x.as_str())?
				.as_f64()
				.ok_or(FetchError::JQLError("X query is null".to_string()))?; // TODO what if it's given to us as a string?
		} else {
			x = Utc::now().timestamp() as f64;
		}
		let y = jql::walker(value, self.query_y.as_str())?
			.as_f64()
			.ok_or(FetchError::JQLError("Y query is null".to_string()))?;
		Ok(PlotPoint { x, y })
	}

	pub fn values(
		&self,
		min_x: Option<f64>,
		max_x: Option<f64>,
		chunk_size: Option<u32>,
		average: bool,
	) -> Vec<PlotPoint> {
		let mut values = self.data.read().expect("PlotPoints RwLock poisoned").clone();
		if let Some(min_x) = min_x {
			values.retain(|x| x.x > min_x);
		}
		if let Some(max_x) = max_x {
			values.retain(|x| x.x < max_x);
		}
		if let Some(chunk_size) = chunk_size {
			if chunk_size > 0 {
				// TODO make this nested if prettier
				let iter = values.chunks(chunk_size as usize);
				values = iter.map(|x| if average { avg_value(x) } else { if x.len() > 0 { x[x.len()-1] } else { PlotPoint {x: 0.0, y:0.0 }} }).collect();
			}
		}
		values
	}
}
