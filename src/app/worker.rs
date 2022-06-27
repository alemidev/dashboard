use crate::app::data::{source::fetch, ApplicationState};
use chrono::Utc;
use eframe::egui::Context;
use std::sync::Arc;
use tracing::warn;

pub fn native_save(state: Arc<ApplicationState>) {
	std::thread::spawn(move || {
		let storage = state.storage.lock().expect("Storage Mutex poisoned");
		let panels = state.panels.read().expect("Panels RwLock poisoned");
		for (index, panel) in panels.iter().enumerate() {
			if let Err(e) = storage.update_panel(
				panel.id,
				panel.name.as_str(),
				panel.view_scroll,
				panel.view_size,
				panel.view_chunks,
				panel.view_offset,
				panel.timeserie,
				panel.width,
				panel.height,
				panel.limit,
				panel.reduce,
				panel.shift,
				index as i32,
			) {
				warn!(target: "native-save", "Could not update panel #{} : {:?}", panel.id, e);
			}
		}
		let sources = state.sources.read().expect("Sources RwLock poisoned");
		for (index, source) in sources.iter().enumerate() {
			if let Err(e) = storage.update_source(
				source.id,
				source.name.as_str(),
				source.enabled,
				source.url.as_str(),
				source.interval,
				index as i32,
			) {
				warn!(target: "native-save", "Could not update source #{} : {:?}", source.id, e);
			}
		}
		let metrics = state.metrics.read().expect("Metrics RwLock poisoned");
		for (index, metric) in metrics.iter().enumerate() {
			if let Err(e) = storage.update_metric(
				metric.id,
				metric.name.as_str(),
				metric.source_id,
				metric.query_x.as_str(),
				metric.query_y.as_str(),
				metric.panel_id,
				metric.color,
				index as i32,
			) {
				warn!(target: "native-save", "Could not update metric #{} : {:?}", metric.id, e);
			}
		}
	});
}

pub(crate) trait BackgroundWorker {
	fn start(state: Arc<ApplicationState>, ctx: Context) -> Self; // TODO make it return an error? Can we even do anything without a background worker
	fn stop(self); // TODO make it return an error? Can we even do anything without a background worker
}

pub(crate) struct NativeBackgroundWorker {
	worker: std::thread::JoinHandle<()>,
}

impl BackgroundWorker for NativeBackgroundWorker {
	fn start(state: Arc<ApplicationState>, ctx: Context) -> Self {
		let worker = std::thread::spawn(move || {
			let mut last_check = 0;
			while state.run {
				let delta_time = 1000 - (Utc::now().timestamp_millis() - last_check);
				if delta_time > 0 {
					std::thread::sleep(std::time::Duration::from_millis(delta_time as u64));
				}
				last_check = Utc::now().timestamp_millis();

				let sources = state.sources.read().expect("Sources RwLock poisoned");
				for j in 0..sources.len() {
					let s_id = sources[j].id;
					if sources[j].enabled && !sources[j].valid() {
						let mut last_update = sources[j]
							.last_fetch
							.write()
							.expect("Sources RwLock poisoned");
						*last_update = Utc::now();
						let state2 = state.clone();
						let source_name = sources[j].name.clone();
						let url = sources[j].url.clone();
						std::thread::spawn(move || {
							// TODO this can overspawn if a request takes longer than the refresh interval!
							match fetch(url.as_str()) {
								Ok(res) => {
									let store =
										state2.storage.lock().expect("Storage mutex poisoned");
									for metric in state2.metrics.read().expect("Metrics RwLock poisoned").iter() {
										if metric.source_id == s_id {
											match metric.extract(&res) {
												Ok(v) => {
													metric.data.write().expect("Data RwLock poisoned").push(v);
													if let Err(e) = store.put_value(metric.id, &v) {
														warn!(target:"background-worker", "Could not put sample for source #{} in db: {:?}", s_id, e);
													}
												}
												Err(e) => {
													warn!(target:"background-worker", "[{}|{}] Could not extract value from result: {:?}", source_name, metric.name, e); // TODO: more info!
												}
											}
										}
									}
									let sources = state2.sources.read().expect("Sources RwLock poisoned");
									let mut last_update = sources[j]
										.last_fetch
										.write()
										.expect("Source last update RwLock poisoned");
									*last_update = Utc::now(); // overwrite it so fetches comply with API slowdowns and get desynched among them
								}
								Err(e) => {
									warn!(target:"background-worker", "Could not fetch value from {} : {:?}", url, e);
								}
							}
						});
					}
				}

				if let Ok(meta) = std::fs::metadata(state.file_path.clone()) {
					let mut fsize = state.file_size.write().expect("File Size RwLock poisoned");
					*fsize = meta.len();
				} // ignore errors

				ctx.request_repaint();
			}
		});

		return NativeBackgroundWorker { worker };
	}

	fn stop(self) {
		self.worker
			.join()
			.expect("Failed joining main worker thread");
	}
}
