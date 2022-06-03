mod app;

pub use app::App;

// When compiling for web:

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
	// Make sure panics are logged using `console.error`.
	console_error_panic_hook::set_once();

	// Redirect tracing to console.log and friends:
	tracing_wasm::set_as_global_default();

	eframe::start_web(canvas_id, Box::new(|cc| Box::new(App::new(cc))))
}
