use std::sync::{Arc, Mutex};
use rand::Rng;
use std::io::{Write, Read};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, de::{DeserializeOwned}};
use eframe::egui::{plot::Value, Context};

pub fn native_save(name: &str, data:String) -> std::io::Result<()> {
	let mut file = std::fs::File::create(name)?;
	file.write_all(data.as_bytes())?;
	return Ok(());
}

pub struct DataSource {
	data : Arc<Mutex<Vec<Value>>>,
}

#[derive(Serialize, Deserialize)]
struct SerializableValue {
	x : f64,
	y : f64,
}

impl DataSource {
	pub fn new() -> Self {
		Self{ data: Arc::new(Mutex::new(Vec::new())) }
	}

	pub fn view(&self) -> Vec<Value> { // TODO handle errors
		return self.data.lock().unwrap().clone();
	}

	pub fn serialize(&self) -> String {
		let mut out : Vec<SerializableValue> = Vec::new();
		for value in self.view() {
			out.push(SerializableValue { x: value.x, y: value.y });
		}
		return serde_json::to_string(&out).unwrap();
	}
}

pub trait PlotValue {
	fn as_value(&self) -> Value;
}

pub trait Data {
	fn load_remote(&mut self, url:&str, ctx:Context);
	fn load_local(&mut self, file:&str, ctx:Context);

	fn read(&mut self, file:&str, storage:Arc<Mutex<Vec<Value>>>, ctx:Context) -> std::io::Result<()> {
		let mut file = std::fs::File::open(file)?;
		let mut contents = String::new();
		file.read_to_string(&mut contents)?;
		let data : Vec<SerializableValue> = serde_json::from_str(contents.as_str())?;
		for v in data {
			storage.lock().unwrap().push(Value { x: v.x, y: v.y });
		}
		ctx.request_repaint();
		Ok(())
	}

	fn fetch<T>(&mut self, base:&str, endpoint:&str, storage:Arc<Mutex<Vec<Value>>>, ctx:Context) 
	where T : DeserializeOwned + PlotValue {
		let request = ehttp::Request::get(format!("{}/{}", base, endpoint));
		ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
			let data : T = serde_json::from_slice(result.unwrap().bytes.as_slice()).unwrap();
			storage.lock().unwrap().push(data.as_value()); 
			ctx.request_repaint();
		});

	}
}

pub struct TpsData {
	pub ds: DataSource,
	load_interval : i64,
	last_load : DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
struct TpsResponseData {
	tps: f64
}

impl PlotValue for TpsResponseData {
	fn as_value(&self) -> Value {
		Value { x: Utc::now().timestamp() as f64, y: self.tps }
	}
}

impl TpsData {
	pub fn new(load_interval:i64) -> Self {
		Self { ds: DataSource::new() , last_load: Utc::now(), load_interval }
	}
}

impl Data for TpsData{
	fn load_remote(&mut self, url:&str, ctx:Context) {
		if (Utc::now() - self.last_load).num_seconds() < self.load_interval { return; }
		self.last_load = Utc::now();
		self.fetch::<TpsResponseData>(url, "tps", self.ds.data.clone(), ctx);
	}

	fn load_local(&mut self, file:&str, ctx:Context) {
		self.read(file, self.ds.data.clone(), ctx).unwrap_or_else(|_err| println!("Could not load {}", file));
	}
}

pub struct ChatData {
	pub ds : DataSource,
	load_interval : i64,
	last_load : DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
struct ChatResponseData {
	volume: f64
}

impl PlotValue for ChatResponseData {
	fn as_value(&self) -> Value {
		Value { x:Utc::now().timestamp() as f64, y: self.volume }
	}
}

impl ChatData {
	pub fn new(load_interval:i64) -> Self {
		Self { ds: DataSource::new() , last_load: Utc::now(), load_interval }
	}
}

impl Data for ChatData{
	fn load_remote(&mut self, url:&str, ctx:Context) {
		if (Utc::now() - self.last_load).num_seconds() < self.load_interval { return; }
		self.last_load = Utc::now();
		self.fetch::<ChatResponseData>(url, "chat_activity", self.ds.data.clone(), ctx);
	}

	fn load_local(&mut self, file:&str, ctx:Context) {
		self.read(file, self.ds.data.clone(), ctx).unwrap_or_else(|_err| println!("Could not load {}", file));
	}
}

pub struct PlayerCountData {
	pub ds : DataSource,
	load_interval : i64,
	last_load : DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
struct PlayerCountResponseData {
	count: i32
}

impl PlotValue for PlayerCountResponseData {
	fn as_value(&self) -> Value {
		Value { x:Utc::now().timestamp() as f64, y: self.count as f64 }
	}
}

impl PlayerCountData {
	pub fn new(load_interval:i64) -> Self {
		Self { ds: DataSource::new() , last_load: Utc::now(), load_interval }
	}
}

impl Data for PlayerCountData{
	fn load_remote(&mut self, url:&str, ctx:Context) {
		if (Utc::now() - self.last_load).num_seconds() < self.load_interval { return; }
		self.last_load = Utc::now();
		self.fetch::<PlayerCountResponseData>(url, "player_count", self.ds.data.clone(), ctx);
	}

	fn load_local(&mut self, file:&str, ctx:Context) {
		self.read(file, self.ds.data.clone(), ctx).unwrap_or_else(|_err| println!("Could not load {}", file));
	}
}

pub struct RandomData {
	pub ds : DataSource,
	load_interval : i64,
	last_load : DateTime<Utc>,
	rng: rand::rngs::ThreadRng,
}

impl RandomData {
	#[allow(dead_code)]
	pub fn new(load_interval:i64) -> Self {
		Self { ds: DataSource::new() , last_load: Utc::now(), load_interval, rng : rand::thread_rng() }
	}
}

impl Data for RandomData{
	fn load_remote(&mut self, _url:&str, ctx:Context) {
		if (Utc::now() - self.last_load).num_seconds() < self.load_interval { return; }
		self.last_load = Utc::now();
		self.ds.data.lock().unwrap().push(Value {x:Utc::now().timestamp() as f64, y:self.rng.gen()}); 
		ctx.request_repaint();
	}

	fn load_local(&mut self, _file:&str, _ctx:Context) {}
}