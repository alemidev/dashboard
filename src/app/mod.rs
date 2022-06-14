pub mod data;
pub mod gui;
pub mod util;
pub mod worker;

use eframe::egui::{
	collapsing_header::CollapsingState, global_dark_light_mode_switch, CentralPanel, Context,
	Layout, ScrollArea, SidePanel, TopBottomPanel,
};
use eframe::emath::Align;
use std::sync::Arc;
use tracing::error;

use self::data::source::{Panel, Source};
use self::data::ApplicationState;
use self::gui::panel::{panel_body_ui, panel_edit_inline_ui, panel_title_ui};
use self::gui::source::source_edit_ui;
use self::util::human_size;
use self::worker::native_save;

pub struct App {
	data: Arc<ApplicationState>,
	input_source: Source,
	input_panel: Panel,
	edit: bool,
	padding: bool,
}

impl App {
	pub fn new(_cc: &eframe::CreationContext, data: Arc<ApplicationState>) -> Self {
		Self {
			data,
			input_panel: Panel::default(),
			input_source: Source::default(),
			edit: false,
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
					for msg in msgs.iter() {
						ui.label(msg);
					}
				});
			});
		});
		if self.edit {
			SidePanel::left("sources-bar")
				.width_range(240.0..=800.0)
				.default_width(500.0)
				.show(ctx, |ui| {
					let panels = self.data.panels.read().expect("Panels RwLock poisoned");
					ScrollArea::vertical().show(ui, |ui| {
						let width = ui.available_width();
						{
							let mut sources = self.data.sources.write().expect("Sources RwLock poisoned");
							for source in &mut *sources {
								source_edit_ui(ui, source, &panels, width);
							}
						}
						ui.add_space(20.0);
						ui.separator();
						ui.horizontal(|ui| {
							ui.heading("new source");
								ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
									ui.horizontal(|ui| {
										if ui.button("add").clicked() {
											if let Err(e) = self.data.add_source(&self.input_source) {
												error!(target: "ui", "Error adding souce : {:?}", e);
											} else {
												self.input_source.id += 1;
											}
										}
										ui.toggle_value(&mut self.padding, "#");
									});
								});
						});
						source_edit_ui(ui, &mut self.input_source, &panels, width);
						if self.padding {
							ui.add_space(300.0);
						}
					});
				});
		}
		let mut to_swap: Vec<usize> = Vec::new();
		CentralPanel::default().show(ctx, |ui| {
			ScrollArea::vertical().show(ui, |ui| {
				let mut panels = self.data.panels.write().expect("Panels RwLock poisoned"); // TODO only lock as write when editing
				let panels_count = panels.len();
				let sources = self.data.sources.read().expect("Sources RwLock poisoned"); // TODO only lock as write when editing
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
									to_swap.push(index); // TODO kinda jank but is there a better way?
								}
							}
							if ui.small_button(" - ").clicked() {
								if index < panels_count - 1 {
									to_swap.push(index + 1); // TODO kinda jank but is there a better way?
								}
							}
							ui.separator();
						}
						panel_title_ui(ui, panel, self.edit);
					})
					.body(|ui| panel_body_ui(ui, panel, &sources));
				}
			});
		});
		if !to_swap.is_empty() {
			// TODO can this be done in background? idk
			let mut panels = self.data.panels.write().expect("Panels RwLock poisoned");
			for index in to_swap {
				panels.swap(index - 1, index);
			}
		}
	}
}
