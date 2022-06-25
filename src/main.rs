mod app;

use crate::app::{
	data::ApplicationState,
	util::InternalLogger,
	worker::{BackgroundWorker, NativeBackgroundWorker},
	App,
};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::metadata::LevelFilter;
use tracing_subscriber::prelude::*;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> ! {
	let native_options = eframe::NativeOptions::default();

	let args: Vec<String> = std::env::args().collect();

	// Set default file location
	let mut store_path = dirs::data_dir().unwrap_or(PathBuf::from(".")); // TODO get cwd more consistently?
	store_path.push("dashboard.db");

	// Can be overruled from argv
	for (index, arg) in args.iter().enumerate() {
		if index <= 0 || arg.eq("--") {
			continue;
		}
		store_path = PathBuf::from(arg.as_str());
		break;
	}

	println!("path: {}", store_path.to_str().unwrap());

	let store =
		Arc::new(ApplicationState::new(store_path).expect("Failed creating application state"));

	tracing_subscriber::registry()
		.with(LevelFilter::INFO)
		.with(tracing_subscriber::fmt::Layer::new())
		.with(InternalLogger::new(store.clone()))
		.init();

	eframe::run_native(
		// TODO replace this with a loop that ends so we can cleanly exit the background worker
		"dashboard",
		native_options,
		Box::new(move |cc| {
			let _worker = NativeBackgroundWorker::start(store.clone(), cc.egui_ctx.clone());
			Box::new(App::new(cc, store))
		}),
	);

	// worker.stop();
}
