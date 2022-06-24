pub mod data;
pub mod gui;
pub mod util;
pub mod worker;

use eframe::egui::Window;
use eframe::egui::{
	collapsing_header::CollapsingState, global_dark_light_mode_switch, CentralPanel, Context,
	Layout, ScrollArea, SidePanel, TopBottomPanel,
};
use eframe::emath::{Align, Pos2};
use std::ops::Index;
use std::sync::Arc;
use tracing::error;

use self::data::source::{Metric, Panel, Source};
use self::data::ApplicationState;
use self::gui::metric::{metric_edit_ui, metric_display_ui};
use self::gui::panel::{panel_body_ui, panel_edit_inline_ui, panel_title_ui};
use self::gui::source::{source_display_ui, source_edit_ui};
use self::util::human_size;
use self::worker::native_save;

pub struct App {
	data: Arc<ApplicationState>,
	input_metric: Metric,
	input_source: Source,
	input_panel: Panel,
	deleting_metric: Option<usize>,
	deleting_source: Option<usize>,
	edit: bool,
	sources: bool,
	padding: bool,
}

impl App {
	pub fn new(_cc: &eframe::CreationContext, data: Arc<ApplicationState>) -> Self {
		Self {
			data,
			input_metric: Metric::default(),
			input_panel: Panel::default(),
			input_source: Source::default(),
			deleting_metric: None,
			deleting_source: None,
			edit: false,
			sources: true,
			padding: false,
		}
	}
}

impl eframe::App for App {
	fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
		TopBottomPanel::top("heading").show(ctx, |ui| {
			ui.horizontal(|ui| {
				global_dark_light_mode_switch(ui);
				ui.heading("dashboard");
				ui.separator();
				ui.checkbox(&mut self.sources, "sources");
				ui.separator();
				ui.checkbox(&mut self.edit, "edit");
				if self.edit {
					if ui.button("save").clicked() {
						native_save(self.data.clone());
						self.edit = false;
					}
					ui.separator();
					ui.label("+ panel");
					panel_edit_inline_ui(ui, &mut self.input_panel);
					if ui.button("add").clicked() {
						if let Err(e) = self.data.add_panel(&self.input_panel) {
							error!(target: "ui", "Failed to add panel: {:?}", e);
						};
					}
					ui.separator();
				}
				ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
					ui.horizontal(|ui| {
						if ui.small_button("×").clicked() {
							frame.quit();
						}
					});
				});
			});
		});
		TopBottomPanel::bottom("footer").show(ctx, |ui| {
			CollapsingState::load_with_default_open(
				ctx,
				ui.make_persistent_id("footer-logs"),
				false,
			)
			.show_header(ui, |ui| {
				ui.horizontal(|ui| {
					ui.separator();
					ui.label(self.data.file_path.to_str().unwrap()); // TODO maybe calculate it just once?
					ui.separator();
					ui.label(human_size(
						*self
							.data
							.file_size
							.read()
							.expect("Filesize RwLock poisoned"),
					));
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
					let msgs = self
						.data
						.diagnostics
						.read()
						.expect("Diagnostics RwLock poisoned");
					ui.separator();
					for msg in msgs.iter() {
						ui.label(msg);
					}
					ui.separator();
				});
			});
		});
		if let Some(index) = self.deleting_metric {
			Window::new(format!("Delete Metric #{}", index))
			.show(ctx, |ui| {
				ui.heading("Are you sure you want to delete this metric?");
				ui.label("This will remove all its metrics and delete all points from archive. This action CANNOT BE UNDONE!");
				ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
					ui.horizontal(|ui| {
						if ui.button("yes").clicked() {
							let store = self.data.storage.lock().expect("Storage Mutex poisoned");
							let mut metrics = self.data.metrics.write().expect("Metrics RwLock poisoned");
							store.delete_metric(metrics[index].id).expect("Failed deleting metric");
							store.delete_values(metrics[index].id).expect("Failed deleting values");
							metrics.remove(index);
							self.deleting_metric = None;
						}
						if ui.button(" no ").clicked() {
							self.deleting_metric = None;
						}
					});
				});
			});
		}
		if let Some(index) = self.deleting_source {
			Window::new(format!("Delete Source #{}", index)).show(ctx, |ui| {
				ui.heading("Are you sure you want to delete this source?");
				ui.label("This will remove all its metrics and delete all points from archive. This action CANNOT BE UNDONE!");
				ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
					ui.horizontal(|ui| {
						if ui.button("YEAH").clicked() {
							let store = self.data.storage.lock().expect("Storage Mutex poisoned");
							let mut sources = self.data.sources.write().expect("sources RwLock poisoned");
							let mut metrics = self.data.metrics.write().expect("Metrics RwLock poisoned");
							let mut to_remove = Vec::new();
							for j in 0..metrics.len() {
								if metrics[j].source_id == self.input_source.id {
									store.delete_values(metrics[j].id).expect("Failed deleting values");
									store.delete_metric(metrics[j].id).expect("Failed deleting Metric");
									to_remove.push(j);
								}
							}
							for index in to_remove {
								metrics.remove(index);
							}
							store.delete_source(sources[index].id).expect("Failed deleting source");
							sources.remove(index);
							self.deleting_source = None;
						}
						if ui.button(" NO WAY ").clicked() {
							self.deleting_source = None;
						}
					});
				});
			});
		}
		if self.sources {
			let mut to_swap: Option<usize> = None;
			// let mut to_delete: Option<usize> = None;
			SidePanel::left("sources-bar")
				.width_range(280.0..=800.0)
				.default_width(330.0)
				.show(ctx, |ui| {
					let panels = self.data.panels.read().expect("Panels RwLock poisoned");
					let panel_width = ui.available_width();
					ScrollArea::both().max_width(panel_width).show(ui, |ui| {
						// TODO only vertical!
						{
							let mut sources =
								self.data.sources.write().expect("Sources RwLock poisoned");
							let sources_count = sources.len();
							ui.heading("Sources");
							ui.separator();
							for (i, source) in sources.iter_mut().enumerate() {
								ui.horizontal(|ui| {
									if self.edit {
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
										if self.edit {
											ui.group(|ui| {
												ui.horizontal(|ui| {
													source_edit_ui(
														ui,
														source,
														remaining_width - 34.0,
													);
													if ui.small_button("×").clicked() {
														self.deleting_metric = None;
														self.deleting_source = Some(i);
													}
												});
												for (j, metric) in self.data.metrics.write().expect("Metrics RwLock poisoned").iter_mut().enumerate() {
													if metric.source_id == source.id {
														ui.horizontal(|ui| {
															metric_edit_ui(ui, metric, Some(&panels), remaining_width - 31.0);
															if ui.small_button("×").clicked() {
																self.deleting_source = None;
																self.deleting_metric = Some(j);
															}
														});
													}
												}
												ui.horizontal(|ui| {
													metric_edit_ui(
														ui,
														&mut self.input_metric,
														None,
														remaining_width - 30.0,
													);
													if ui.small_button("           +          ").clicked() { // TODO find a better
														if let Err(e) = self
															.data
															.add_metric(&self.input_metric, source)
														{
															error!(target: "ui", "Error adding metric : {:?}", e);
														}
													}
													ui.add_space(1.0); // DAMN!
													if ui.small_button("×").clicked() {
														self.input_metric = Metric::default();
													}
												})
											});
										} else {
											let metrics =
												self.data.metrics.read().expect("Metrics RwLock poisoned");
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
						if self.edit {
							ui.separator();
							ui.horizontal(|ui| {
								ui.heading("new source");
								ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
									ui.horizontal(|ui| {
										if ui.button("add").clicked() {
											if let Err(e) = self.data.add_source(&self.input_source) {
												error!(target: "ui", "Error adding source : {:?}", e);
											} else {
												self.input_source.id += 1;
											}
										}
										ui.toggle_value(&mut self.padding, "#");
									});
								});
							});
							source_edit_ui(
								ui,
								&mut self.input_source,
								panel_width - 10.0,
							);
							ui.add_space(5.0);
							if self.padding {
								ui.add_space(300.0);
							}
						}
					});
				});
			//if let Some(i) = to_delete {
			//	// TODO can this be done in background? idk
			//	let mut panels = self.data.panels.write().expect("Panels RwLock poisoned");
			//	panels.remove(i);
			// } else
			if let Some(i) = to_swap {
				// TODO can this be done in background? idk
				let mut sources = self.data.sources.write().expect("Sources RwLock poisoned");
				sources.swap(i - 1, i);
			}
		}
		let mut to_swap: Option<usize> = None;
		let mut to_delete: Option<usize> = None;
		CentralPanel::default().show(ctx, |ui| {
			ScrollArea::vertical().show(ui, |ui| {
				let mut panels = self.data.panels.write().expect("Panels RwLock poisoned"); // TODO only lock as write when editing
				let panels_count = panels.len();
				let metrics = self.data.metrics.read().expect("Metrics RwLock poisoned"); // TODO only lock as write when editing
				for (index, panel) in panels.iter_mut().enumerate() {
					if index > 0 {
						ui.separator();
					}
					CollapsingState::load_with_default_open(
						ctx,
						ui.make_persistent_id(format!("panel-{}-compressable", panel.id)),
						true,
					)
					.show_header(ui, |ui| {
						if self.edit {
							if ui.small_button(" + ").clicked() {
								if index > 0 {
									to_swap = Some(index); // TODO kinda jank but is there a better way?
								}
							}
							if ui.small_button(" − ").clicked() {
								if index < panels_count - 1 {
									to_swap = Some(index + 1); // TODO kinda jank but is there a better way?
								}
							}
							if ui.small_button(" × ").clicked() {
								to_delete = Some(index); // TODO kinda jank but is there a better way?
							}
							ui.separator();
						}
						panel_title_ui(ui, panel, self.edit);
					})
					.body(|ui| panel_body_ui(ui, panel, &metrics));
				}
			});
		});
		if let Some(i) = to_delete {
			// TODO can this be done in background? idk
			let mut panels = self.data.panels.write().expect("Panels RwLock poisoned");
			if let Err(e) = self
				.data
				.storage
				.lock()
				.expect("Storage Mutex poisoned")
				.delete_panel(panels[i].id)
			{
				error!(target: "ui", "Could not delete panel : {:?}", e);
			} else {
				for metric in self
					.data
					.metrics
					.write()
					.expect("Sources RwLock poisoned")
					.iter_mut()
				{
					if metric.panel_id == panels[i].id {
						metric.panel_id = -1;
					}
				}
				panels.remove(i);
			}
		} else if let Some(i) = to_swap {
			// TODO can this be done in background? idk
			let mut panels = self.data.panels.write().expect("Panels RwLock poisoned");
			panels.swap(i - 1, i);
		}
	}
}
