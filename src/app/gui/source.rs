use eframe::{egui::{Ui, TextEdit, DragValue, Checkbox, ScrollArea, Layout}, emath::Align};
use tracing::error;

use crate::app::{data::source::{Source, Metric}, App};

use super::metric::{metric_edit_ui, metric_display_ui};

pub fn source_panel(app: &mut App, ui: &mut Ui) {
	let mut to_swap: Option<usize> = None;
	// let mut to_delete: Option<usize> = None;
	let panels = app.data.panels.read().expect("Panels RwLock poisoned");
	let panel_width = ui.available_width();
	ScrollArea::both().max_width(panel_width).show(ui, |ui| {
		// TODO only vertical!
		{
			let mut sources =
				app.data.sources.write().expect("Sources RwLock poisoned");
			let sources_count = sources.len();
			ui.heading("Sources");
			ui.separator();
			for (i, source) in sources.iter_mut().enumerate() {
				ui.horizontal(|ui| {
					if app.edit {
						ui.vertical(|ui| {
							ui.add_space(10.0);
							if ui.small_button("+").clicked() {
								if i > 0 {
									to_swap = Some(i); // TODO kinda jank but is there a better way?
								}
							}
							if ui.small_button("−").clicked() {
								if i < sources_count - 1 {
									to_swap = Some(i + 1); // TODO kinda jank but is there a better way?
								}
							}
						});
					}
					ui.vertical(|ui| {
						let remaining_width = ui.available_width();
						if app.edit {
							ui.group(|ui| {
								ui.horizontal(|ui| {
									source_edit_ui(
										ui,
										source,
										remaining_width - 34.0,
									);
									if ui.small_button("×").clicked() {
										app.deleting_metric = None;
										app.deleting_source = Some(i);
									}
								});
								for (j, metric) in app.data.metrics.write().expect("Metrics RwLock poisoned").iter_mut().enumerate() {
									if metric.source_id == source.id {
										ui.horizontal(|ui| {
											metric_edit_ui(ui, metric, Some(&panels), remaining_width - 31.0);
											if ui.small_button("×").clicked() {
												app.deleting_source = None;
												app.deleting_metric = Some(j);
											}
										});
									}
								}
								ui.horizontal(|ui| {
									metric_edit_ui(
										ui,
										&mut app.input_metric,
										None,
										remaining_width - 30.0,
									);
									if ui.small_button("           +          ").clicked() { // TODO find a better
										if let Err(e) = app
											.data
											.add_metric(&app.input_metric, source)
										{
											error!(target: "ui", "Error adding metric : {:?}", e);
										}
									}
									ui.add_space(1.0); // DAMN!
									if ui.small_button("×").clicked() {
										app.input_metric = Metric::default();
									}
								})
							});
						} else {
							let metrics =
								app.data.metrics.read().expect("Metrics RwLock poisoned");
							source_display_ui(
								ui,
								source,
								remaining_width,
							);
							for metric in metrics.iter() {
								if metric.source_id == source.id {
									metric_display_ui(ui, metric, ui.available_width());
								}
							}
							ui.separator();
						}
					});
				});
			}
		}
		if app.edit {
			ui.separator();
			ui.horizontal(|ui| {
				ui.heading("new source");
				ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
					ui.horizontal(|ui| {
						if ui.button("add").clicked() {
							if let Err(e) = app.data.add_source(&app.input_source) {
								error!(target: "ui", "Error adding source : {:?}", e);
							} else {
								app.input_source.id += 1;
							}
						}
						ui.toggle_value(&mut app.padding, "#");
					});
				});
			});
			source_edit_ui(
				ui,
				&mut app.input_source,
				panel_width - 10.0,
			);
			ui.add_space(5.0);
			if app.padding {
				ui.add_space(300.0);
			}
		}
	});
	//if let Some(i) = to_delete {
	//	// TODO can this be done in background? idk
	//	let mut panels = app.data.panels.write().expect("Panels RwLock poisoned");
	//	panels.remove(i);
	// } else
	if let Some(i) = to_swap {
		// TODO can this be done in background? idk
		let mut sources = app.data.sources.write().expect("Sources RwLock poisoned");
		sources.swap(i - 1, i);
	}
}

pub fn source_display_ui(ui: &mut Ui, source: &mut Source, _width: f32) {
	ui.horizontal(|ui| {
		ui.add_enabled(false, Checkbox::new(&mut source.enabled, ""));
		ui.add_enabled(false, DragValue::new(&mut source.interval).clamp_range(1..=120));
		ui.heading(&source.name).on_hover_text(&source.url);
	});
}

pub fn source_edit_ui(ui: &mut Ui, source: &mut Source, width: f32) {
	ui.horizontal(|ui| {
		let text_width = width - 100.0;
		ui.checkbox(&mut source.enabled, "");
		ui.add(DragValue::new(&mut source.interval).clamp_range(1..=3600));
		TextEdit::singleline(&mut source.name)
			.desired_width(text_width / 4.0)
			.hint_text("name")
			.show(ui);
		TextEdit::singleline(&mut source.url)
			.desired_width(text_width * 3.0 / 4.0)
			.hint_text("url")
			.show(ui);
	});
}
