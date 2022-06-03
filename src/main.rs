mod app;

use app::App;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
	let native_options = eframe::NativeOptions::default();

	eframe::run_native(
		"2b2t queue stats",
		native_options,
		Box::new(|cc| Box::new(App::new(cc))),
	);
}
