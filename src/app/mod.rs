pub mod data;

use std::sync::{Arc, Mutex};
use chrono::{DateTime, NaiveDateTime, Utc};
use data::source::{ChatData, PlayerCountData, TpsData, Data, native_save};
use eframe::egui;
use eframe::egui::plot::{Line, Plot, Values};
use crate::app::data::store::DataStorage;

use self::data::store::SQLiteDataStore;

pub struct App {
	// data : SQLiteDataStore,
	data : Arc<SQLiteDataStore>,
}

struct ServerOptions {
	title: String,
	url: String,
	player_count: PlayerCountData,
	tps: TpsData,
	chat: ChatData,
	sync_time:bool,
}

impl App {
	// pub fn new(_cc: &eframe::CreationContext, data: SQLiteDataStore) -> Self {
	pub fn new(_cc: &eframe::CreationContext, data: Arc<SQLiteDataStore>) -> Self {
		Self { data }
	}
}

impl eframe::App for App {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		egui::TopBottomPanel::top("??? wtf").show(ctx, |ui| {
			ui.horizontal(|ui| {
				egui::widgets::global_dark_light_mode_switch(ui);
				ui.heading("dashboard");
				if ui.button("test add").clicked() {
					self.data.add_panel("test panel");
				}
				ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
					if ui.button("x").clicked() {
						frame.quit();
					}
				});
			});
		});
		egui::CentralPanel::default().show(ctx, |ui| {
			let panels = &*self.data.panels.lock().unwrap();
			for i in 0..panels.len() {
			// for panel in self.data.view() {
				ui.group(|ui| {
					ui.vertical(|ui| {
						ui.horizontal(|ui| {
							ui.heading(panels[i].name.as_str());
							// ui.checkbox(&mut panel.view_scroll, "autoscroll");
						});
						let mut p = Plot::new(format!("plot-{}", panels[i].name)).x_axis_formatter(|x, _range| {
							format!(
								"{}",
								DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(x as i64, 0), Utc)
									.format("%Y/%m/%d %H:%M:%S")
							)
						}).center_x_axis(false).height(panels[i].height as f32); // TODO make it fucking reactive! It fills the whole screen with 1 plot no matter what I do...

						if panels[i].view_scroll {
							p = p.include_x(Utc::now().timestamp() as f64);
						}

						p.show(ui, |plot_ui| {
							let sources = &*panels[i].sources.lock().unwrap();
							for j in 0..sources.len() {
								plot_ui.line(Line::new(sources[j].values()).name(sources[j].name.as_str()));
							}
						});
					});
				});
			}
		});
		ctx.request_repaint(); // TODO super jank way to sorta keep drawing
	}
}
