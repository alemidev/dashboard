use eframe::egui::{ScrollArea, Ui, DragValue, TextEdit, Checkbox};

use crate::gui::App;
use crate::data::entities;

use super::metric::metric_edit_ui;

pub fn source_panel(app: &mut App, ui: &mut Ui) {
	let source_to_put_metric_on : Option<i64> = None;
	// let mut to_delete: Option<usize> = None;
	let panel_width = ui.available_width();
	let mut orphaned_metrics = app.view.metrics.borrow().clone();
	ScrollArea::vertical()
		.max_width(panel_width)
		.show(ui, |ui| {
			// TODO only vertical!
			{
				let sources = app.view.sources.borrow();
				ui.heading("Sources");
				ui.separator();
				for source in sources.iter() {
					ui.add_space(5.0);
					ui.horizontal(|ui| {
						ui.vertical(|ui| {
							ui.add_space(8.0);
							if ui.small_button("#").clicked() {
								app.editing.push(source.clone().into());
							}
							// if ui.small_button("−").clicked() { }
						});
						ui.vertical(|ui| { // actual sources list container
							ui.group(|ui| {
								ui.horizontal(|ui| {
									source_edit_ui(ui, source);
								});
								let metrics = app
									.view
									.metrics
									.borrow();
								for (_j, metric) in metrics.iter().enumerate() {
									if metric.source_id == source.id {
										orphaned_metrics.retain(|m| m.id != metric.id);
										ui.horizontal(|ui| {
											metric_edit_ui(ui, metric);
											// if ui.small_button("s").clicked() {
											// 	let path = FileDialog::new()
											// 		.add_filter("csv", &["csv"])
											// 		.set_file_name(format!("{}-{}.csv", source.name, metric.name).as_str())
											// 		.save_file();
											// 	if let Some(_path) = path {
											// 		// serialize_values(
											// 		// 	&*metric
											// 		// 		.data
											// 		// 		.read()
											// 		// 		.expect("Values RwLock poisoned"),
											// 		// 	metric,
											// 		// 	path,
											// 		// )
											// 		// .expect("Could not serialize data");
											// 	}
											// }
											if ui.small_button("#").clicked() {
												// TODO don't add duplicates
												app.editing.push(metric.clone().into());
											}
										});
									}
								}
							});
						});
					});
				}
				ui.add_space(5.0);
				ui.horizontal(|ui| { // 1 more for uncategorized sources
					ui.vertical(|ui| {
						ui.add_space(8.0);
						if ui.small_button("+").clicked() {
							app.editing.push(entities::sources::Model::default().into());
						}
					});
					ui.vertical(|ui| { // actual sources list container
						ui.group(|ui| {
							ui.horizontal(|ui| {
								source_edit_ui(ui, &app.buffer_source);
							});
							for metric in orphaned_metrics.iter() {
								ui.horizontal(|ui| {
									metric_edit_ui(ui, metric);
									// if ui.small_button("s").clicked() {
									// 	// let path = FileDialog::new()
									// 	// 	.add_filter("csv", &["csv"])
									// 	// 	.set_file_name(format!("{}-{}.csv", source.name, metric.name).as_str())
									// 	// 	.save_file();
									// 	// if let Some(_path) = path {
									// 	// 	// serialize_values(
									// 	// 	// 	&*metric
									// 	// 	// 		.data
									// 	// 	// 		.read()
									// 	// 	// 		.expect("Values RwLock poisoned"),
									// 	// 	// 	metric,
									// 	// 	// 	path,
									// 	// 	// )
									// 	// 	// .expect("Could not serialize data");
									// 	// }
									// }
									if ui.small_button("#").clicked() {
										// TODO don't add duplicates
										app.editing.push(metric.clone().into());
									}
								});
							}
							// Add an empty metric to insert new ones
							ui.horizontal(|ui| {
								metric_edit_ui(ui, &mut app.buffer_metric);
								if ui.small_button("+").clicked() {
									app.editing.push(entities::metrics::Model::default().into());
								}
							});
						});
					});
				});
			}
		});
	//if let Some(i) = to_delete {
	//	// TODO can this be done in background? idk
	//	let mut panels = app.data.panels.write().expect("Panels RwLock poisoned");
	//	panels.remove(i);
	// } else
	// if let Some(i) = to_swap {
	// 	// TODO can this be done in background? idk
	// 	let mut sources = app.sources.borrow();
	// 	sources.swap(i - 1, i);
	// }
	// if to_insert.len() > 0 {
	// 	let mut metrics = app.metrics.borrow();
	// 	for m in to_insert {
	// 		metrics.push(m);
	// 	}
	// }
	if let Some(s) = source_to_put_metric_on {
		for source in app.view.sources.borrow().iter() {
			if source.id == s {
				// if let Err(e) =
				// 	app.data.add_metric(&app.input_metric, &source)
				// {
				// 	error!(target: "ui", "Error adding metric : {:?}", e);
				// }
			}
		}
	}
}

pub fn _source_display_ui(ui: &mut Ui, source: &entities::sources::Model, _width: f32) {
	ui.horizontal(|ui| {
		ui.heading(&source.name).on_hover_text(&source.url);
	// 	ui.add_enabled(false, Checkbox::new(&mut source.enabled, ""));
	});
}

pub fn source_edit_ui(ui: &mut Ui, source: &entities::sources::Model) {
	let mut interval = source.interval.clone();
	let mut name = source.name.clone();
	let mut enabled = source.enabled.clone();
	ui.horizontal(|ui| {
		ui.add_enabled(false, Checkbox::new(&mut enabled, ""));
		TextEdit::singleline(&mut name)
			.desired_width(ui.available_width() - 58.0)
			.interactive(false)
			.hint_text("name")
			.show(ui);
		ui.add_enabled(false, DragValue::new(&mut interval).clamp_range(1..=3600));
	});
}
