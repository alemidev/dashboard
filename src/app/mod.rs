pub mod data;
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
use self::util::{human_size, timestamp_to_str};
use self::worker::native_save;

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
}

impl App {
	pub fn new(_cc: &eframe::CreationContext, data: Arc<ApplicationState>) -> Self {
		Self {
			data,
			input: InputBuffer::default(),
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
					eframe::egui::TextEdit::singleline(&mut self.input.panel_name)
						.hint_text("name")
						.desired_width(50.0)
						.show(ui);
					if ui.button("add").clicked() {
						if let Err(e) = self.data.add_panel(self.input.panel_name.as_str()) {
							error!(target: "ui", "Failed to add panel: {:?}", e);
						};
					}
					ui.separator();
					ui.label("+ source");
					eframe::egui::TextEdit::singleline(&mut self.input.name)
						.hint_text("name")
						.desired_width(50.0)
						.show(ui);
					eframe::egui::TextEdit::singleline(&mut self.input.url)
						.hint_text("url")
						.desired_width(160.0)
						.show(ui);
					eframe::egui::TextEdit::singleline(&mut self.input.query_x)
						.hint_text("x")
						.desired_width(30.0)
						.show(ui);
					eframe::egui::TextEdit::singleline(&mut self.input.query_y)
						.hint_text("y")
						.desired_width(30.0)
						.show(ui);
					egui::ComboBox::from_id_source("panel")
						.selected_text(format!("panel [{}]", self.input.panel_id))
						.width(70.0)
						.show_ui(ui, |ui| {
							let pnls = self.data.panels.write().expect("Panels RwLock poisoned");
							for p in &*pnls {
								ui.selectable_value(
									&mut self.input.panel_id,
									p.id,
									p.name.as_str(),
								);
							}
						});
					ui.checkbox(&mut self.input.visible, "visible");
					ui.add(egui::Slider::new(&mut self.input.interval, 1..=60));
					ui.color_edit_button_srgba(&mut self.input.color);
					if ui.button("add").clicked() {
						if let Err(e) = self.data.add_source(
							self.input.panel_id,
							self.input.name.as_str(),
							self.input.url.as_str(),
							self.input.query_x.as_str(),
							self.input.query_y.as_str(),
							self.input.color,
							self.input.visible,
						) {
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
				egui::ScrollArea::vertical().show(ui, |ui| {
					for source in &mut *sources {
						ui.group(|ui| {
							ui.horizontal(|ui| {
								ui.checkbox(&mut source.visible, "");
								eframe::egui::TextEdit::singleline(&mut source.name)
									.hint_text("name")
									.desired_width(80.0)
									.show(ui);
								eframe::egui::TextEdit::singleline(&mut source.url)
									.hint_text("url")
									.desired_width(300.0)
									.show(ui);
							});
							ui.horizontal(|ui| {
								ui.add(egui::Slider::new(&mut source.interval, 1..=60));
								eframe::egui::TextEdit::singleline(&mut source.query_x)
									.hint_text("x")
									.desired_width(50.0)
									.show(ui);
								eframe::egui::TextEdit::singleline(&mut source.query_y)
									.hint_text("y")
									.desired_width(50.0)
									.show(ui);
								egui::ComboBox::from_id_source(format!("panel-{}", source.id))
									.selected_text(format!("panel [{}]", source.panel_id))
									.width(70.0)
									.show_ui(ui, |ui| {
										let pnls = self
											.data
											.panels
											.read()
											.expect("Panels RwLock poisoned");
										for p in &*pnls {
											ui.selectable_value(
												&mut source.panel_id,
												p.id,
												p.name.as_str(),
											);
										}
									});
								ui.color_edit_button_srgba(&mut source.color);
							});
						});
					}
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
						ui.horizontal(|ui| {
							if self.edit {
								if ui.small_button(" ^ ").clicked() {
									if index > 0 {
										to_swap.push(index); // TODO kinda jank but is there a better way?
									}
								}
								ui.separator();
							}
							ui.heading(panel.name.as_str());
							if self.edit {
								ui.separator();
								ui.add(
									egui::Slider::new(&mut panel.height, 0..=500).text("height"),
								);
								ui.separator();
								ui.checkbox(&mut panel.timeserie, "timeserie");
							}
							ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
								ui.horizontal(|ui| {
									ui.toggle_value(&mut panel.view_scroll, " • ");
									ui.separator();
									ui.label("m");
									ui.add(
										egui::DragValue::new(&mut panel.view_size)
											.speed(10)
											.clamp_range(0..=2147483647i32),
									);
									ui.checkbox(&mut panel.limit, "limit");
								});
							});
						});
					})
					.body(|ui| {
						let mut p = Plot::new(format!("plot-{}", panel.name))
							.height(panel.height as f32)
							.allow_scroll(false)
							.legend(egui::plot::Legend::default().position(egui::plot::Corner::LeftTop));

						if panel.view_scroll {
							p = p
								.include_x(Utc::now().timestamp() as f64);
							if panel.limit {
								p = p.include_x(
									(Utc::now().timestamp() - (panel.view_size as i64 * 60))
										as f64,
								);
							}
						}

						if panel.timeserie {
							p = p
								.x_axis_formatter(|x, _range| {
									timestamp_to_str(x as i64, true, false)
								})
								.label_formatter(|name, value| {
									if !name.is_empty() {
										return format!(
											"{}\nx = {}\ny = {:.1}",
											name,
											timestamp_to_str(value.x as i64, false, true),
											value.y
										);
									} else {
										return format!(
											"x = {}\ny = {:.1}",
											timestamp_to_str(value.x as i64, false, true),
											value.y
										);
									}
								})
								.x_grid_spacer(|grid| {
									let offset = Local::now().offset().local_minus_utc() as i64;
									let (start, end) = grid.bounds;
									let mut counter = (start as i64) - ((start as i64) % 3600);
									let mut out: Vec<GridMark> = Vec::new();
									loop {
										counter += 3600;
										if counter > end as i64 {
											break;
										}
										if (counter + offset) % 86400 == 0 {
											out.push(GridMark {
												value: counter as f64,
												step_size: 86400 as f64,
											})
										} else if counter % 3600 == 0 {
											out.push(GridMark {
												value: counter as f64,
												step_size: 3600 as f64,
											});
										}
									}
									return out;
								});
						}

						p.show(ui, |plot_ui| {
							for source in &*sources {
								if source.visible && source.panel_id == panel.id {
									let line = if panel.limit {
										Line::new(source.values_filter(
											(Utc::now().timestamp()
												- (panel.view_size as i64 * 60)) as f64,
										))
										.name(source.name.as_str())
									} else {
										Line::new(source.values()).name(source.name.as_str())
									};
									plot_ui.line(line.color(source.color));
								}
							}
						});
					});
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
