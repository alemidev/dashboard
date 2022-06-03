mod datasource;

use chrono::{DateTime, NaiveDateTime, Utc};
use datasource::{ChatData, PlayerCountData, TpsData, Data, RandomData};
use eframe::egui;
use eframe::egui::plot::{Line, Plot, Value, Values};


pub struct App {
	player_count: PlayerCountData,
	tps: TpsData,
	chat: ChatData,
	rand: RandomData,
	sync_time:bool,
}

impl App {
	pub fn new(_cc: &eframe::CreationContext) -> Self {
		Self {
			player_count: PlayerCountData::new(60),
			tps: TpsData::new(30),
			chat: ChatData::new(15),
			rand: RandomData::new(1),
			sync_time: false,
		}
	}
}

impl eframe::App for App {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		self.rand.load("");
		egui::TopBottomPanel::top("??? wtf").show(ctx, |ui| {
			ui.horizontal(|ui| {
				egui::widgets::global_dark_light_mode_switch(ui);
				ui.heading("nnbot dashboard");
				ui.checkbox(&mut self.sync_time, "Lock X to now");
				ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
					if ui.button("x").clicked() {
						frame.quit();
					}
				});
			});
		});
		egui::CentralPanel::default().show(ctx, |ui| {
			let mut p = Plot::new("test").x_axis_formatter(|x, _range| {
				format!(
					"{}",
					DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(x as i64, 0), Utc)
						.format("%Y/%m/%d %H:%M:%S")
				)
			}).center_x_axis(false);

			if self.sync_time {
				p = p.include_x(Utc::now().timestamp() as f64);
			}

			p.show(ui, |plot_ui| {
				plot_ui.line(
					Line::new(Values::from_values(self.player_count.view())).name("Player Count"),
				);
				plot_ui.line(Line::new(Values::from_values(self.tps.view())).name("TPS over 15s"));
				plot_ui.line(Line::new(Values::from_values(self.rand.view())).name("Random Data"));
				plot_ui.line(
					Line::new(Values::from_values(self.chat.view()))
						.name("Chat messages per minute"),
				);
			});
		});
	}
}
