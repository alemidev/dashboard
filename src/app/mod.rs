pub mod data;
pub mod worker;

use std::sync::Arc;
use chrono::{DateTime, NaiveDateTime, Utc};
use eframe::egui;
use eframe::egui::{plot::{Line, Plot}};

use self::data::ApplicationState;
use self::worker::native_save;

struct InputBuffer {
	panel_name: String,
	name: String,
	url: String,
	interval: i32,
	query_x: String,
	query_y: String,
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
			panel_id: 0,
		}	
	}
}

pub struct App {
	data: Arc<ApplicationState>,
	input: InputBuffer,
	edit: bool,
}

impl App {
	pub fn new(_cc: &eframe::CreationContext, data: Arc<ApplicationState>) -> Self {
		Self { data, input: InputBuffer::default(), edit: false }
	}
}

impl eframe::App for App {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		egui::TopBottomPanel::top("??? wtf").show(ctx, |ui| {
			ui.horizontal(|ui| {
				egui::widgets::global_dark_light_mode_switch(ui);
				ui.heading("dashboard");
				ui.checkbox(&mut self.edit, "edit");
				if self.edit {
					if ui.button("save").clicked() {
						native_save(self.data.clone());
					}
				}
				ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
					if ui.button("×").clicked() {
						frame.quit();
					}
				});
			});
			if self.edit {
				ui.horizontal(|ui| {
					eframe::egui::TextEdit::singleline(&mut self.input.panel_name).hint_text("panel").desired_width(50.0).show(ui);
					if ui.button("add panel").clicked() {
						self.data.add_panel(self.input.panel_name.as_str()).unwrap();
					}
					eframe::egui::TextEdit::singleline(&mut self.input.name).hint_text("name").desired_width(30.0).show(ui);
					eframe::egui::TextEdit::singleline(&mut self.input.url).hint_text("url").desired_width(80.0).show(ui);
					eframe::egui::TextEdit::singleline(&mut self.input.query_x).hint_text("x query").desired_width(25.0).show(ui);
					eframe::egui::TextEdit::singleline(&mut self.input.query_y).hint_text("y query").desired_width(25.0).show(ui);
					egui::ComboBox::from_label("panel")
						.selected_text(format!("[{}]", self.input.panel_id))
						.show_ui(ui, |ui| {
							let pnls = self.data.panels.write().unwrap();
							for p in &*pnls {
								ui.selectable_value(&mut self.input.panel_id, p.id, p.name.as_str());
							}
						}
					);
					if ui.button("add source").clicked() {
						self.data.add_source(
							self.input.panel_id,
							self.input.name.as_str(),
							self.input.url.as_str(),
							self.input.query_x.as_str(),
							self.input.query_y.as_str(),
						).unwrap();
					}
					ui.add(egui::Slider::new(&mut self.input.interval, 1..=600).text("interval"));
				});
			}
		});
		egui::CentralPanel::default().show(ctx, |ui| {
			egui::ScrollArea::vertical().show(ui, |ui| {
				let mut panels = self.data.panels.write().unwrap();
				for panel in &mut *panels {
				// for panel in self.data.view() {
					ui.group(|ui| {
						ui.vertical(|ui| {
							ui.horizontal(|ui| {
								ui.heading(panel.name.as_str());
								ui.checkbox(&mut panel.view_scroll, "autoscroll");
								ui.checkbox(&mut panel.timeserie, "timeserie");
								ui.add(egui::Slider::new(&mut panel.height, 0..=500).text("height"));
							});

							let mut sources = panel.sources.write().unwrap();

							if self.edit {
								for source in &mut *sources {
									ui.horizontal(|ui| {
										ui.heading(source.name.as_str());
										eframe::egui::TextEdit::singleline(&mut source.url).hint_text("url").show(ui);
										eframe::egui::TextEdit::singleline(&mut source.query_x).hint_text("x query").show(ui);
										eframe::egui::TextEdit::singleline(&mut source.query_y).hint_text("y query").show(ui);
										ui.add(egui::Slider::new(&mut source.interval, 1..=600).text("interval"));
									});
								}
							}

							let mut p = Plot::new(format!("plot-{}", panel.name))
								.height(panel.height as f32); // TODO make it fucking reactive! It fills the whole screen with 1 plot no matter what I do...

							if panel.view_scroll {
								p = p.include_x(Utc::now().timestamp() as f64);
							}

							if panel.timeserie {
								p = p.x_axis_formatter(|x, _range| {
									format!(
										"{}",
										DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(x as i64, 0), Utc)
											.format("%Y/%m/%d %H:%M:%S")
									)
								});
							}

							p.show(ui, |plot_ui| {
								for source in &mut *sources {
									plot_ui.line(Line::new(source.values()).name(source.name.as_str()));
								}
							});
						});
					});
				}
			});
		});
	}
}
