use eframe::{
	egui::{Checkbox, DragValue, Layout, ScrollArea, TextEdit, Ui},
	emath::Align, epaint::Color32,
};
use rfd::FileDialog;
use tracing::error;

use crate::app::{
	data::source::{Metric, Source},
	util::{deserialize_values, serialize_values},
	App,
};

use super::metric::{metric_display_ui, metric_edit_ui};

pub fn source_panel(app: &mut App, ui: &mut Ui) {
	let mut source_to_put_metric_on : Option<i32> = None;
	let mut to_swap: Option<usize> = None;
	let mut to_insert: Vec<Metric> = Vec::new();
	// let mut to_delete: Option<usize> = None;
	let panels = app.data.panels.read().expect("Panels RwLock poisoned");
	let panel_width = ui.available_width();
	ScrollArea::vertical()
		.max_width(panel_width)
		.show(ui, |ui| {
			// TODO only vertical!
			{
				let mut sources = app.data.sources.write().expect("Sources RwLock poisoned");
				let sources_count = sources.len();
				ui.heading("Sources");
				ui.separator();
				for (i, source) in sources.iter_mut().enumerate() {
					ui.horizontal(|ui| {
						if app.edit { // show buttons to move sources up and down
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
						ui.vertical(|ui| { // actual sources list container
							let remaining_width = ui.available_width();
							if app.edit {
								ui.group(|ui| {
									ui.horizontal(|ui| {
										source_edit_ui(ui, source, remaining_width - 34.0);
										if ui.small_button("×").clicked() {
											app.deleting_metric = None;
											app.deleting_source = Some(i);
										}
									});
									let mut metrics = app
										.data
										.metrics
										.write()
										.expect("Metrics RwLock poisoned");
									for (j, metric) in metrics.iter_mut().enumerate() {
										if metric.source_id == source.id {
											ui.horizontal(|ui| {
												metric_edit_ui(
													ui,
													metric,
													Some(&panels),
													remaining_width - 53.0,
												);
												if ui.small_button("s").clicked() {
													let path = FileDialog::new()
														.add_filter("csv", &["csv"])
														.set_file_name(format!("{}-{}.csv", source.name, metric.name).as_str())
														.save_file();
													if let Some(path) = path {
														serialize_values(
															&*metric
																.data
																.read()
																.expect("Values RwLock poisoned"),
															metric,
															path,
														)
														.expect("Could not serialize data");
													}
												}
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
											remaining_width - 53.0,
										);
										ui.add_space(2.0);
										if ui.small_button("          +          ").clicked() {
											source_to_put_metric_on = Some(source.id);
										}
										ui.add_space(1.0); // DAMN!
										if ui.small_button("o").clicked() {
											let path = FileDialog::new()
												.add_filter("csv", &["csv"])
												.pick_file();
											if let Some(path) = path {
												match deserialize_values(path) {
													Ok((name, query_x, query_y, data)) => {
														let mut store = app
															.data
															.storage
															.lock()
															.expect("Storage Mutex poisoned");
														match store.new_metric(
															name.as_str(),
															source.id,
															query_x.as_str(),
															query_y.as_str(),
															-1,
															Color32::TRANSPARENT,
															metrics.len() as i32,
														) {
															Ok(verified_metric) => {
																store.put_values(verified_metric.id, &data).unwrap();
																*verified_metric.data.write().expect("Values RwLock poisoned") = data;
																to_insert.push(verified_metric);
															}
															Err(e) => {
																error!(target: "ui", "could not save metric into archive : {:?}", e);
															}
														}
													}
													Err(e) => {
														error!(target: "ui", "Could not deserialize metric from file : {:?}", e);
													}
												}
											}
										}
										if ui.small_button("×").clicked() {
											app.input_metric = Metric::default();
										}
									})
								});
							} else {
								let metrics =
									app.data.metrics.read().expect("Metrics RwLock poisoned");
								source_display_ui(ui, source, remaining_width);
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
				source_edit_ui(ui, &mut app.input_source, panel_width - 10.0);
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
	if to_insert.len() > 0 {
		let mut metrics = app.data.metrics.write().expect("Metrics RwLock poisoned");
		for m in to_insert {
			metrics.push(m);
		}
	}
	if let Some(s) = source_to_put_metric_on {
		for source in app.data.sources.read().expect("Sources RwLock poisoned").iter() {
			if source.id == s {
				if let Err(e) =
					app.data.add_metric(&app.input_metric, &source)
				{
					error!(target: "ui", "Error adding metric : {:?}", e);
				}
			}
		}
	}
}

pub fn source_display_ui(ui: &mut Ui, source: &mut Source, _width: f32) {
	ui.horizontal(|ui| {
		ui.add_enabled(false, Checkbox::new(&mut source.enabled, ""));
		ui.add_enabled(
			false,
			DragValue::new(&mut source.interval).clamp_range(1..=120),
		);
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
