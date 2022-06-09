pub mod data;
pub mod worker;
pub mod util;

use std::sync::Arc;
use chrono::Utc;
use eframe::egui;
use eframe::egui::{RichText, plot::{Line, Plot}, Color32};

use self::data::ApplicationState;
use self::worker::native_save;
use self::util::{human_size, timestamp_to_str};

struct InputBuffer {
	panel_name: String,
	name: String,
	url: String,
	interval: i32,
	query_x: String,
	query_y: String,
	color: Color32,
	visible: bool,
	panel_id: i32,
}

impl Default for InputBuffer {
	fn default() -> Self {
		InputBuffer {
			panel_name: "".to_string(),
			name: "".to_string(),
			url: "".to_string(),
			interval: 60,
			query_x: "".to_string(),
			query_y: "".to_string(),
			color: Color32::TRANSPARENT,
			visible: true,
			panel_id: 0,
		}	
	}
}

pub struct App {
	data: Arc<ApplicationState>,
	input: InputBuffer,
	edit: bool,
	filter: bool,
}

impl App {
	pub fn new(_cc: &eframe::CreationContext, data: Arc<ApplicationState>) -> Self {
		Self { data, input: InputBuffer::default(), edit: false, filter: false }
	}
}

impl eframe::App for App {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		egui::TopBottomPanel::top("heading").show(ctx, |ui| {
			ui.horizontal(|ui| {
				egui::widgets::global_dark_light_mode_switch(ui);
				ui.heading("dashboard");
				ui.separator();
				ui.checkbox(&mut self.filter, "filter");
				ui.separator();
				ui.checkbox(&mut self.edit, "edit");
				if self.edit {
					if ui.button("save").clicked() {
						native_save(self.data.clone());
					}
					ui.separator();
					ui.label("+ panel");
					eframe::egui::TextEdit::singleline(&mut self.input.panel_name).hint_text("name").desired_width(50.0).show(ui);
					if ui.button("add").clicked() {
						self.data.add_panel(self.input.panel_name.as_str()).unwrap();
					}
					ui.separator();
					ui.label("+ source");
					eframe::egui::TextEdit::singleline(&mut self.input.name).hint_text("name").desired_width(35.0).show(ui);
					eframe::egui::TextEdit::singleline(&mut self.input.url).hint_text("url").desired_width(80.0).show(ui);
					eframe::egui::TextEdit::singleline(&mut self.input.query_x).hint_text("x").desired_width(25.0).show(ui);
					eframe::egui::TextEdit::singleline(&mut self.input.query_y).hint_text("y").desired_width(25.0).show(ui);
					egui::ComboBox::from_id_source("panel")
						.selected_text(format!("panel [{}]", self.input.panel_id))
						.width(70.0)
						.show_ui(ui, |ui| {
							let pnls = self.data.panels.write().unwrap();
							for p in &*pnls {
								ui.selectable_value(&mut self.input.panel_id, p.id, p.name.as_str());
							}
						}
					);
					ui.checkbox(&mut self.input.visible, "visible");
					ui.add(egui::Slider::new(&mut self.input.interval, 1..=60));
					ui.color_edit_button_srgba(&mut self.input.color);
					if ui.button("add").clicked() {
						self.data.add_source(
							self.input.panel_id,
							self.input.name.as_str(),
							self.input.url.as_str(),
							self.input.query_x.as_str(),
							self.input.query_y.as_str(),
							self.input.color,
							self.input.visible,
						).unwrap();
					}
					ui.separator();
				}
				ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
					if ui.button("×").clicked() {
						frame.quit();
					}
				});
			});
		});
		egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
			ui.horizontal(|ui|{
				ui.label(self.data.file_path.to_str().unwrap());
				ui.separator();
				ui.label(human_size(*self.data.file_size.read().unwrap()));
				ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
					ui.horizontal(|ui| {
						ui.label(format!("v{}-{}", env!("CARGO_PKG_VERSION"), git_version::git_version!()));
						ui.separator();
						ui.hyperlink_to("<me@alemi.dev>", "mailto:me@alemi.dev");
						ui.label("alemi");
					});
				});
			});
		});
		if self.edit {
			egui::SidePanel::left("sources-bar").show(ctx, |ui| {
				let mut sources = self.data.sources.write().expect("Sources RwLock poisoned");
				egui::ScrollArea::vertical().show(ui, |ui| {
					for source in &mut *sources {
						ui.group(|ui| {
							ui.horizontal(|ui| {
								ui.checkbox(&mut source.visible, "");
								eframe::egui::TextEdit::singleline(&mut source.name).hint_text("name").desired_width(80.0).show(ui);
								eframe::egui::TextEdit::singleline(&mut source.url).hint_text("url").desired_width(300.0).show(ui);
							});
							ui.horizontal(|ui| {
								ui.add(egui::Slider::new(&mut source.interval, 1..=60));
								eframe::egui::TextEdit::singleline(&mut source.query_x).hint_text("x").desired_width(50.0).show(ui);
								eframe::egui::TextEdit::singleline(&mut source.query_y).hint_text("y").desired_width(50.0).show(ui);
								egui::ComboBox::from_id_source(format!("panel-{}", source.id))
									.selected_text(format!("panel [{}]", source.panel_id))
									.width(70.0)
									.show_ui(ui, |ui| {
										let pnls = self.data.panels.read().expect("Panels RwLock poisoned");
										for p in &*pnls {
											ui.selectable_value(&mut source.panel_id, p.id, p.name.as_str());
										}
									});
								ui.color_edit_button_srgba(&mut source.color);
							});
						});
					}
				});
			});
		}
		egui::CentralPanel::default().show(ctx, |ui| {
			egui::ScrollArea::vertical().show(ui, |ui| {
				let mut panels = self.data.panels.write().unwrap(); // TODO only lock as write when editing
				let sources = self.data.sources.read().unwrap(); // TODO only lock as write when editing
				for panel in &mut *panels {
					ui.group(|ui| {
						ui.vertical(|ui| {
							ui.horizontal(|ui| {
								ui.heading(panel.name.as_str());
								ui.separator();
								for source in &*sources {
									if source.panel_id == panel.id {
										if source.visible {
											ui.label(
												RichText::new(source.name.as_str())
												.color(if source.color == Color32::TRANSPARENT { Color32::GRAY } else { source.color })
											);
										} else {
											ui.label(RichText::new(source.name.as_str()).color(Color32::BLACK));
										}
										ui.separator();
									}
								}
								if self.filter {
									ui.add(egui::Slider::new(&mut panel.view_size, 1..=1440).text("samples"));
									ui.separator();
								}
								ui.add(egui::Slider::new(&mut panel.height, 0..=500).text("height"));
								ui.separator();
								ui.checkbox(&mut panel.view_scroll, "autoscroll");
								ui.checkbox(&mut panel.timeserie, "timeserie");
								ui.separator();
							});

							let mut p = Plot::new(format!("plot-{}", panel.name))
								.height(panel.height as f32)
								.allow_scroll(false);

							if panel.view_scroll {
								p = p.include_x(Utc::now().timestamp() as f64);
								if self.filter {
									p = p.include_x((Utc::now().timestamp() - (panel.view_size as i64 * 60)) as f64);
								}
							}

							if panel.timeserie {
								p = p.x_axis_formatter(|x, _range| timestamp_to_str(x as i64));
								p = p.label_formatter(|name, value| {
									if !name.is_empty() {
										return format!("{}\nx = {}\ny = {:.1}", name, timestamp_to_str(value.x as i64), value.y)
									} else {
										return format!("x = {}\ny = {:.1}", timestamp_to_str(value.x as i64), value.y);
									}
								});
							}

							p.show(ui, |plot_ui| {
								for source in &*sources {
									if source.visible && source.panel_id == panel.id {
										let line = if self.filter {
											Line::new(source.values_filter((Utc::now().timestamp() - (panel.view_size as i64 * 60)) as f64)).name(source.name.as_str())
										} else {
											Line::new(source.values()).name(source.name.as_str())
										};
										plot_ui.line(line.color(source.color));
									}
								}
							});
						});
					});
				}
			});
		});
	}
}
