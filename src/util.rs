use chrono::{DateTime, Local, NaiveDateTime, Utc};
use eframe::egui::{Color32, plot::PlotPoint};
use tokio::sync::{watch, mpsc};
use tracing::error;
use std::{error::Error, path::PathBuf, collections::VecDeque};
use tracing_subscriber::Layer;

use super::data::entities;

// if you're handling more than terabytes of data, it's the future and you ought to update this code!
const _PREFIXES: &'static [&'static str] = &["", "k", "M", "G", "T"];

pub fn _serialize_values(values: &Vec<PlotPoint>, metric: &entities::metrics::Model, path: PathBuf) -> Result<(), Box<dyn Error>> {
	let mut wtr = csv::Writer::from_writer(std::fs::File::create(path)?);
	// DAMN!   VVVVV
	let name = metric.name.as_str();
	let q = metric.query.as_str();
	wtr.write_record(&[name, q, "1"])?;
	// DAMN!   AAAAA
	for v in values {
		wtr.serialize(("", v.x, v.y))?;
	}
	wtr.flush()?;
	Ok(())
}

pub fn _deserialize_values(path: PathBuf) -> Result<(String, String, String, Vec<PlotPoint>), Box<dyn Error>> {
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

#[allow(dead_code)]
pub fn human_size(size: u64) -> String {
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

pub fn unpack_color(c: i32) -> Color32 {
	let r: u8 = (c >> 0) as u8;
	let g: u8 = (c >> 8) as u8;
	let b: u8 = (c >> 16) as u8;
	let a: u8 = (c >> 24) as u8;
	return Color32::from_rgba_unmultiplied(r, g, b, a);
}

#[allow(dead_code)]
pub fn repack_color(c: Color32) -> i32 {
	let mut out: i32 = 0;
	let mut offset = 0;
	for el in c.to_array() {
		out |= ((el & 0xFF) as i32) << offset;
		offset += 8;
	}
	return out;
}

pub struct InternalLogger {
	size: usize,
	view_tx: watch::Sender<Vec<String>>,
	view_rx: watch::Receiver<Vec<String>>,
	msg_tx : mpsc::UnboundedSender<String>,
	msg_rx : mpsc::UnboundedReceiver<String>,
}

impl InternalLogger {
	pub fn new(size: usize) -> Self {
		let (view_tx, view_rx) = watch::channel(vec![]);
		let (msg_tx, msg_rx) = mpsc::unbounded_channel();
		InternalLogger { size, view_tx, view_rx, msg_tx, msg_rx }
	}

	pub fn view(&self) -> watch::Receiver<Vec<String>> {
		self.view_rx.clone()
	}

	pub fn layer(&self) -> InternalLoggerLayer {
		InternalLoggerLayer::new(self.msg_tx.clone())
	}

	pub async fn worker(mut self, run: watch::Receiver<bool>) {
		let mut messages = VecDeque::new();
		while *run.borrow() {
			tokio::select!{
				msg = self.msg_rx.recv() => {
					match msg {
						Some(msg) => {
							messages.push_back(msg);
							while messages.len() > self.size {
								messages.pop_front();
							}
							if let Err(e) = self.view_tx.send(messages.clone().into()) {
								error!(target: "internal-logger", "Failed sending log line: {:?}", e);
							}
						},
						None => break,
					}
				},
				_ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {},
				// unblock so it checks again run and exits cleanly
			}
		}
	}
}

pub struct InternalLoggerLayer {
	msg_tx:  mpsc::UnboundedSender<String>,
}

impl InternalLoggerLayer {
	pub fn new(msg_tx: mpsc::UnboundedSender<String>) -> Self {
		InternalLoggerLayer { msg_tx }
	}
}

impl<S> Layer<S> for InternalLoggerLayer
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
		let out = format!(
			"{} [{}] {}: {}",
			Local::now().format("%H:%M:%S"),
			event.metadata().level(),
			event.metadata().target(),
			msg_visitor.msg
		);

		self.msg_tx.send(out).unwrap_or_default();
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
