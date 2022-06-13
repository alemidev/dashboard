pub mod data;
pub mod gui;
pub mod util;
pub mod worker;

use chrono::{Local, Utc};
use eframe::egui;
use eframe::egui::plot::GridMark;
use eframe::egui::{
	plot::{Line, Plot},
	Color32,
};
use std::sync::Arc;
use tracing::error;

use self::data::ApplicationState;
use self::data::source::{Panel,Source};
use self::gui::panel::{panel_edit_inline_ui, panel_title_ui, panel_body_ui};
use self::gui::source::{source_ui, source_edit_inline_ui};
use self::util::{human_size, timestamp_to_str};
use self::worker::native_save;

pub struct App {
	data: Arc<ApplicationState>,
	input_source: Source,
	input_panel: Panel,
	edit: bool,
}

impl App {
	pub fn new(_cc: &eframe::CreationContext, data: Arc<ApplicationState>) -> Self {
		Self {
			data,
			input_panel: Panel::default(),
			input_source: Source::default(),
			edit: false,
		}
	}
}

impl eframe::App for App {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		egui::TopBottomPanel::top("heading").show(ctx, |ui| {
			ui.horizontal(|ui| {
				egui::widgets::global_dark_light_mode_switch(ui);
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
					ui.label("+ source");
					source_edit_inline_ui(ui, &mut self.input_source, &self.data.panels.read().expect("Panels RwLock poisoned"));
					if ui.button("add").clicked() {
						if let Err(e) = self.data.add_source(&self.input_source) {
							error!(target: "ui", "Error adding souce : {:?}", e);
						}
					}
					ui.separator();
				}
				ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
					ui.horizontal(|ui| {
						if ui.small_button("×").clicked() {
							frame.quit();
						}
					});
				});
			});
		});
		egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
			egui::collapsing_header::CollapsingState::load_with_default_open(
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
					ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
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
				egui::ScrollArea::vertical().show(ui, |ui| {
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
			egui::SidePanel::left("sources-bar").show(ctx, |ui| {
				let mut sources = self.data.sources.write().expect("Sources RwLock poisoned");
				let panels = self.data.panels.read().expect("Panels RwLock poisoned");
				egui::ScrollArea::vertical().show(ui, |ui| {
					for source in &mut *sources {
						source_ui(ui, source, &panels);
					}
					// TODO make this not necessary
					ui.collapsing("extra space", |ui| {
						ui.add_space(300.0);
						ui.separator();
					})
				});
			});
		}
		let mut to_swap: Vec<usize> = Vec::new();
		egui::CentralPanel::default().show(ctx, |ui| {
			egui::ScrollArea::vertical().show(ui, |ui| {
				let mut panels = self.data.panels.write().expect("Panels RwLock poisoned"); // TODO only lock as write when editing
				let panels_count = panels.len();
				let sources = self.data.sources.read().expect("Sources RwLock poisoned"); // TODO only lock as write when editing
				for (index, panel) in panels.iter_mut().enumerate() {
					if index > 0 {
						ui.separator();
					}
					egui::collapsing_header::CollapsingState::load_with_default_open(
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
							}
							panel_title_ui(ui, panel);
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
