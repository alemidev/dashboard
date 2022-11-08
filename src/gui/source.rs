use eframe::egui::{ScrollArea, Ui, DragValue, TextEdit, Checkbox};

use crate::gui::App;
use crate::data::entities;

use super::metric::metric_line_ui;

pub fn source_panel_ui(app: &mut App, ui: &mut Ui) {
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
						});
						ui.vertical(|ui| { // actual sources list container
							ui.group(|ui| {
								ui.horizontal(|ui| {
									source_line_ui(ui, source);
								});
								let metrics = app
									.view
									.metrics
									.borrow();
								for (_j, metric) in metrics.iter().enumerate() {
									if metric.source_id == source.id {
										orphaned_metrics.retain(|m| m.id != metric.id);
										ui.horizontal(|ui| {
											metric_line_ui(ui, metric);
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
								source_line_ui(ui, &app.buffer_source);
							});
							for metric in orphaned_metrics.iter() {
								ui.horizontal(|ui| {
									metric_line_ui(ui, metric);
									if ui.small_button("#").clicked() {
										// TODO don't add duplicates
										app.editing.push(metric.clone().into());
									}
								});
							}
							// Add an empty metric to insert new ones
							ui.horizontal(|ui| {
								metric_line_ui(ui, &mut app.buffer_metric);
								if ui.small_button("+").clicked() {
									app.editing.push(entities::metrics::Model::default().into());
								}
							});
						});
					});
				});
			}
		});
}

pub fn source_line_ui(ui: &mut Ui, source: &entities::sources::Model) {
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
