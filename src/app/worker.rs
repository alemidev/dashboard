use std::sync::Arc;
use chrono::Utc;
use eframe::egui::Context;
use crate::app::data::{fetch, ApplicationState};

pub fn native_save(state:Arc<ApplicationState>) {
	std::thread::spawn(move || {
		let storage = state.storage.lock().unwrap();
		let panels = state.panels.read().unwrap();
		for panel in &*panels {
			storage.update_panel(
				panel.id,
				panel.name.as_str(),
				panel.view_scroll,
				panel.view_size,
				panel.timeserie,
				panel.width,
				panel.height
			).unwrap();
			let sources = panel.sources.read().unwrap();
			for source in &*sources {
				storage.update_source(
					source.id,
					source.name.as_str(),
					source.url.as_str(),
					source.interval,
					source.query_x.as_str(),
					source.query_y.as_str(),
				).unwrap();
			}
		}
	});
}

pub(crate) trait BackgroundWorker {
	fn start(state:Arc<ApplicationState>, ctx:Context) -> Self;  // TODO make it return an error? Can we even do anything without a background worker
	fn stop(self);   // TODO make it return an error? Can we even do anything without a background worker
}

pub(crate) struct NativeBackgroundWorker {
	worker : std::thread::JoinHandle<()>,
}

impl BackgroundWorker for NativeBackgroundWorker {
	fn start(state:Arc<ApplicationState>, ctx:Context) -> Self {
		let worker = std::thread::spawn(move || {
			let mut last_check = 0;
			while state.run {
				let delta_time = 1000 - (Utc::now().timestamp_millis() - last_check);
				if delta_time > 0 {
					std::thread::sleep(std::time::Duration::from_millis(delta_time as u64));
				}
				last_check = Utc::now().timestamp_millis();

				let panels = state.panels.read().unwrap();
				for i in 0..panels.len() {
					let sources = panels[i].sources.read().unwrap();
					let p_id = panels[i].id;
					for j in 0..sources.len() {
						let s_id = sources[j].id;
						if !sources[j].valid() {
							let mut last_update = sources[j].last_fetch.write().unwrap();
							*last_update = Utc::now();
							let state2 = state.clone();
							let url = sources[j].url.clone();
							let query_x = sources[j].query_x.clone();
							let query_y = sources[j].query_y.clone();
							std::thread::spawn(move || { // TODO this can overspawn if a request takes longer than the refresh interval!
								let v = fetch(url.as_str(), query_x.as_str(), query_y.as_str()).unwrap();
								let store = state2.storage.lock().unwrap();
								store.put_value(p_id, s_id, v).unwrap();
								let panels = state2.panels.read().unwrap();
								let sources = panels[i].sources.read().unwrap();
								sources[j].data.write().unwrap().push(v);
								let mut last_update = sources[j].last_fetch.write().unwrap();
								*last_update = Utc::now(); // overwrite it so fetches comply with API slowdowns and get desynched among them
							});
						}
					}
				}

				let mut fsize = state.file_size.write().expect("File Size RwLock poisoned");
				*fsize = std::fs::metadata(state.file_path.clone()).unwrap().len();

				ctx.request_repaint();
			}
		});

		return NativeBackgroundWorker {
			worker
		};
	}

	fn stop(self) {
		self.worker.join().expect("Failed joining main worker thread");
	}
}