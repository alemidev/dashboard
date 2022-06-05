mod app;
mod util;

use app::App;

use crate::util::worker::{BackgroundWorker, NativeBackgroundWorker};

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
	let native_options = eframe::NativeOptions::default();

	let worker = NativeBackgroundWorker::start();

	eframe::run_native( // TODO replace this with a loop that ends so we can cleanly exit the background worker
		"2b2t queue stats",
		native_options,
		Box::new(|cc| Box::new(App::new(cc))),
	);

	// worker.stop();
}
