use eframe::{Frame, egui::{collapsing_header::CollapsingState, Context, Ui, Layout, ScrollArea, global_dark_light_mode_switch}, emath::Align};
use tokio::sync::watch;

use crate::gui::App;

use super::panel::panel_edit_inline_ui;

// TODO make this not super specific!
pub fn _confirmation_popup_delete_metric(_app: &mut App, ui: &mut Ui, _metric_index: usize) {
	ui.heading("Are you sure you want to delete this metric?");
	ui.label("This will remove all its metrics and delete all points from archive. This action CANNOT BE UNDONE!");
	ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
		ui.horizontal(|ui| {
			if ui.button("\n   yes   \n").clicked() {
				// let store = app.data.storage.lock().expect("Storage Mutex poisoned");
				// let mut metrics = app.data.metrics.write().expect("Metrics RwLock poisoned");
				// store.delete_metric(metrics[metric_index].id).expect("Failed deleting metric");
				// store.delete_values(metrics[metric_index].id).expect("Failed deleting values");
				// metrics.remove(metric_index);
				// app.deleting_metric = None;
			}
			if ui.button("\n   no    \n").clicked() {
				// app.deleting_metric = None;
			}
		});
	});
}

// TODO make this not super specific!
pub fn _confirmation_popup_delete_source(_app: &mut App, ui: &mut Ui, _source_index: usize) {
	ui.heading("Are you sure you want to delete this source?");
	ui.label("This will remove all its metrics and delete all points from archive. This action CANNOT BE UNDONE!");
	ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
		ui.horizontal(|ui| {
			if ui.button("\n   yes   \n").clicked() {
				// let store = app.data.storage.lock().expect("Storage Mutex poisoned");
				// let mut sources = app.data.sources.write().expect("sources RwLock poisoned");
				// let mut metrics = app.data.metrics.write().expect("Metrics RwLock poisoned");
				// let mut to_remove = Vec::new();
				// for j in 0..metrics.len() {
				// 	if metrics[j].source_id == app.input_source.id {
				// 		store.delete_values(metrics[j].id).expect("Failed deleting values");
				// 		store.delete_metric(metrics[j].id).expect("Failed deleting Metric");
				// 		to_remove.push(j);
				// 	}
				// }
				// for index in to_remove {
				// 	metrics.remove(index);
				// }
				// store.delete_source(sources[source_index].id).expect("Failed deleting source");
				// sources.remove(source_index);
				// app.deleting_source = None;
			}
			if ui.button("\n   no    \n").clicked() {
				// app.deleting_source = None;
			}
		});
	});
}

pub fn header(app: &mut App, ui: &mut Ui, frame: &mut Frame) {
	ui.horizontal(|ui| {
		global_dark_light_mode_switch(ui);
		ui.heading("dashboard");
		ui.separator();
		ui.checkbox(&mut app.sidebar, "sources");
		ui.separator();
		if ui.button("reset").clicked() {
			app.panels = app.view.panels.borrow().clone();
		}
		ui.separator();
		if ui.button("save").clicked() {
			app.save_all_panels();
		}
		ui.separator();
		if ui.button("refresh").clicked() {
			app.refresh_data();
		}
		ui.separator();
		ui.checkbox(&mut app.edit, "edit");
		if app.edit {
			ui.label("+ panel");
			panel_edit_inline_ui(ui, &mut app.buffer_panel);
			if ui.button("add").clicked() {
				// if let Err(e) = app.data.add_panel(&app.input_panel) {
				// 	error!(target: "ui", "Failed to add panel: {:?}", e);
				// };
			}
			ui.separator();
		}
		ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
			ui.horizontal(|ui| {
				if ui.small_button("×").clicked() {
					frame.close();
				}
			});
		});
	});
}

pub fn footer(ctx: &Context, ui: &mut Ui, diagnostics: watch::Receiver<Vec<String>>, db_path: String, records: usize) {
	CollapsingState::load_with_default_open(
		ctx,
		ui.make_persistent_id("footer-logs"),
		false,
	)
	.show_header(ui, |ui| {
		ui.horizontal(|ui| {
			ui.separator();
			ui.label(db_path); // TODO maybe calculate it just once?
			ui.separator();
			ui.label(format!("{} records loaded", records)); // TODO put thousands separator
			// ui.label(human_size(
			// 	*data
			// 		.file_size
			// 		.read()
			// 		.expect("Filesize RwLock poisoned"),
			// ));
			ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
				ui.horizontal(|ui| {
					ui.label(format!(
						"v{}-{}",
						env!("CARGO_PKG_VERSION"),
						git_version::git_version!()
					));
					ui.separator();
					ui.hyperlink_to("<me@alemi.dev>", "mailto:me@alemi.dev");
					ui.label("alemi");
				});
			});
		});
	})
	.body(|ui| {
		ui.set_height(200.0);
		ScrollArea::vertical().show(ui, |ui| {
			ui.separator();
			for msg in diagnostics.borrow().iter() {
				ui.label(msg);
			}
			ui.separator();
		});
	});
}
