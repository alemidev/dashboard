mod app;
mod util;

use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
use crate::util::worker::{BackgroundWorker, NativeBackgroundWorker};
use crate::app::{App, data::store::{SQLiteDataStore, DataStorage}};

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> ! {
	let native_options = eframe::NativeOptions::default();
	
	let mut store_path = dirs::data_dir().unwrap_or(std::path::PathBuf::from(".")); // TODO get cwd more consistently?
	store_path.push("dashboard.db");

	println!("{}", store_path.as_path().to_str().unwrap());

	let store = Arc::new(
			SQLiteDataStore::new(store_path)
				.unwrap()
	);

	eframe::run_native( // TODO replace this with a loop that ends so we can cleanly exit the background worker
		"dashboard",
		native_options,
		Box::new(move |cc| {
			let worker = NativeBackgroundWorker::start();
			let ctx = cc.egui_ctx.clone();
			worker.task(async move {
				loop {
					sleep(Duration::from_secs(1)).await;
					ctx.request_repaint();
					// tokio::spawn(async move {store2.fetch_all().await});
				}
			});

			Box::new(App::new(cc, store))
		}),
	);

	// worker.stop();
}
