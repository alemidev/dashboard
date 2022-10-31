use chrono::{DateTime, Local, NaiveDateTime, Utc};
use eframe::egui::{Color32, plot::PlotPoint};
use std::{sync::Arc, error::Error, path::PathBuf};
use tracing_subscriber::Layer;

use super::data::{ApplicationState, entities};

// if you're handling more than terabytes of data, it's the future and you ought to update this code!
const _PREFIXES: &'static [&'static str] = &["", "k", "M", "G", "T"];

pub fn _serialize_values(values: &Vec<PlotPoint>, metric: &entities::metrics::Model, path: PathBuf) -> Result<(), Box<dyn Error>> {
	let mut wtr = csv::Writer::from_writer(std::fs::File::create(path)?);
	// DAMN!   VVVVV
	let def_q_x = "".into();
	let name = metric.name.as_str();
	let q_x = metric.query_x.as_ref().unwrap_or(&def_q_x).as_str();
	let q_y = metric.query_y.as_str();
	wtr.write_record(&[name, q_x, q_y])?;
	// DAMN!   AAAAA
	for v in values {
		wtr.serialize(("", v.x, v.y))?;
	}
	wtr.flush()?;
	Ok(())
}

pub fn deserialize_values(path: PathBuf) -> Result<(String, String, String, Vec<PlotPoint>), Box<dyn Error>> {
	let mut values = Vec::new();

	let mut rdr = csv::Reader::from_reader(std::fs::File::open(path)?);
	let mut name = "N/A".to_string();
	let mut query_x = "".to_string();
	let mut query_y = "".to_string();
	if rdr.has_headers() {
		let record = rdr.headers()?;
		name = record[0].to_string();
		query_x = record[1].to_string();
		query_y = record[2].to_string();
	}
	for result in rdr.records() {
		if let Ok(record) = result {
			values.push(PlotPoint { x: record[1].parse::<f64>()?, y: record[2].parse::<f64>()? });
		}
	}

	Ok((
		name,
		query_x,
		query_y,
		values,
	))
}

pub fn _human_size(size: u64) -> String {
	let mut buf: f64 = size as f64;
	let mut prefix: usize = 0;
	while buf > 1024.0 && prefix < _PREFIXES.len() - 1 {
		buf /= 1024.0;
		prefix += 1;
	}

	return format!("{:.3} {}B", buf, _PREFIXES[prefix]);
}

pub fn timestamp_to_str(t: i64, date: bool, time: bool) -> String {
	format!(
		"{}",
		DateTime::<Local>::from(DateTime::<Utc>::from_utc(
			NaiveDateTime::from_timestamp(t, 0),
			Utc
		))
		.format(if date && time {
			"%Y/%m/%d %H:%M:%S"
		} else if date {
			"%Y/%m/%d"
		} else if time {
			"%H:%M:%S"
		} else {
			"%s"
		})
	)
}

pub fn unpack_color(c: u32) -> Color32 {
	let r: u8 = (c >> 0) as u8;
	let g: u8 = (c >> 8) as u8;
	let b: u8 = (c >> 16) as u8;
	let a: u8 = (c >> 24) as u8;
	return Color32::from_rgba_unmultiplied(r, g, b, a);
}

#[allow(dead_code)]
pub fn repack_color(c: Color32) -> u32 {
	let mut out: u32 = 0;
	let mut offset = 0;
	for el in c.to_array() {
		out |= ((el & 0xFF) as u32) << offset;
		offset += 8;
	}
	return out;
}

pub struct InternalLogger {
	_state: Arc<ApplicationState>,
}

impl InternalLogger {
	pub fn _new(state: Arc<ApplicationState>) -> Self {
		InternalLogger { _state: state }
	}
}

impl<S> Layer<S> for InternalLogger
where
	S: tracing::Subscriber,
{
	fn on_event(
		&self,
		event: &tracing::Event<'_>,
		_ctx: tracing_subscriber::layer::Context<'_, S>,
	) {
		let mut msg_visitor = LogMessageVisitor {
			msg: "".to_string(),
		};
		event.record(&mut msg_visitor);
		let _out = format!(
			"{} [{}] {}: {}",
			Local::now().format("%H:%M:%S"),
			event.metadata().level(),
			event.metadata().target(),
			msg_visitor.msg
		);
		// self.state
		// 	.diagnostics
		// 	.write()
		// 	.expect("Diagnostics RwLock poisoned")
		// 	.push(out);
	}
}

struct LogMessageVisitor {
	msg: String,
}

impl tracing::field::Visit for LogMessageVisitor {
	fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
		if field.name() == "message" {
			self.msg = format!("{}: '{:?}' ", field.name(), &value);
		}
	}

	fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
		if field.name() == "message" {
			self.msg = value.to_string();
		}
	}
}
