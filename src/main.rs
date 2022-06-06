mod app;

use std::sync::Arc;
use crate::app::{App, data::ApplicationState, worker::{BackgroundWorker, NativeBackgroundWorker}};

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> ! {
	let native_options = eframe::NativeOptions::default();
	
	let mut store_path = dirs::data_dir().unwrap_or(std::path::PathBuf::from(".")); // TODO get cwd more consistently?
	store_path.push("dashboard.db");

	let store = Arc::new(ApplicationState::new(store_path));

	eframe::run_native( // TODO replace this with a loop that ends so we can cleanly exit the background worker
		"dashboard",
		native_options,
		Box::new(move |cc| {
			let _worker = NativeBackgroundWorker::start(store.clone(), cc.egui_ctx.clone());
			Box::new(App::new(cc, store))
		}),
	);

	// worker.stop();
}
