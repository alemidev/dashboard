use std::sync::{Arc, Mutex};
use rand::Rng;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use eframe::egui::plot::Value;

struct DataSource {
	data : Arc<Mutex<Vec<Value>>>,
}

impl DataSource {
	fn new() -> Self {
		Self{ data: Arc::new(Mutex::new(Vec::new())) }
	}

	fn view(&self) -> Vec<Value> { // TODO handle errors
		return self.data.lock().unwrap().clone();
	}
}

pub trait Data {
	fn load(&mut self, url:&str);
	fn view(&self) -> Vec<Value>;
}

pub struct TpsData {
	ds: DataSource,
	load_interval : i64,
	last_load : DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
struct TpsResponseData {
	tps: f64
}

impl TpsData {
	pub fn new(load_interval:i64) -> Self {
		Self { ds: DataSource::new() , last_load: Utc::now(), load_interval }
	}
}

impl Data for TpsData{
	fn load(&mut self, url:&str) {
		if (Utc::now() - self.last_load).num_seconds() < self.load_interval { return; }
		self.last_load = Utc::now();
		let ds_data = self.ds.data.clone();
		let request = ehttp::Request::get(format!("{}/tps", url));
		ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
			let data : TpsResponseData = serde_json::from_slice(result.unwrap().bytes.as_slice()).unwrap();
			ds_data.lock().unwrap().push(Value {x:Utc::now().timestamp() as f64, y:data.tps}); 
		});
	}

	fn view(&self) -> Vec<Value> { self.ds.view() }
}

pub struct ChatData {
	ds : DataSource,
	load_interval : i64,
	last_load : DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
struct ChatResponseData {
	volume: f64
}

impl ChatData {
	pub fn new(load_interval:i64) -> Self {
		Self { ds: DataSource::new() , last_load: Utc::now(), load_interval }
	}
}

impl Data for ChatData{
	fn load(&mut self, url:&str) {
		if (Utc::now() - self.last_load).num_seconds() < self.load_interval { return; }
		self.last_load = Utc::now();
		let ds_data = self.ds.data.clone();
		let request = ehttp::Request::get(format!("{}/chat_activity", url));
		ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
			let data : ChatResponseData = serde_json::from_slice(result.unwrap().bytes.as_slice()).unwrap();
			ds_data.lock().unwrap().push(Value {x:Utc::now().timestamp() as f64, y:data.volume}); 
		});
	}

	fn view(&self) -> Vec<Value> { self.ds.view() }
}

pub struct PlayerCountData {
	ds : DataSource,
	load_interval : i64,
	last_load : DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
struct PlayerCountResponseData {
	count: i32
}

impl PlayerCountData {
	pub fn new(load_interval:i64) -> Self {
		Self { ds: DataSource::new() , last_load: Utc::now(), load_interval }
	}
}

impl Data for PlayerCountData{
	fn load(&mut self, url:&str) {
		if (Utc::now() - self.last_load).num_seconds() < self.load_interval { return; }
		self.last_load = Utc::now();
		let ds_data = self.ds.data.clone();
		let request = ehttp::Request::get(format!("{}/chat_activity", url));
		ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
			let data : PlayerCountResponseData = serde_json::from_slice(result.unwrap().bytes.as_slice()).unwrap();
			ds_data.lock().unwrap().push(Value {x:Utc::now().timestamp() as f64, y:data.count as f64}); 
		});
	}

	fn view(&self) -> Vec<Value> { self.ds.view() }
}

pub struct RandomData {
	ds : DataSource,
	load_interval : i64,
	last_load : DateTime<Utc>,
	rng: rand::rngs::ThreadRng,
}

impl RandomData {
	pub fn new(load_interval:i64) -> Self {
		Self { ds: DataSource::new() , last_load: Utc::now(), load_interval, rng : rand::thread_rng() }
	}
}

impl Data for RandomData{
	fn load(&mut self, _url:&str) {
		if (Utc::now() - self.last_load).num_seconds() < self.load_interval { return; }
		self.last_load = Utc::now();
		self.ds.data.lock().unwrap().push(Value {x:Utc::now().timestamp() as f64, y:self.rng.gen()}); 
	}

	fn view(&self) -> Vec<Value> { self.ds.view() }
}