mod datasource;

use chrono::{DateTime, NaiveDateTime, Utc};
use datasource::{ChatData, PlayerCountData, TpsData, Data,  native_save};
use eframe::egui;
use eframe::egui::plot::{Line, Plot, Values};

pub struct App {
	servers : Vec<ServerOptions>,
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
	pub fn new(_cc: &eframe::CreationContext) -> Self {
		let mut servers = Vec::new();
		servers.push(ServerOptions::new("9b9t", "https://alemi.dev/mcbots/9b"));
		servers.push(ServerOptions::new("const", "https://alemi.dev/mcbots/const"));
		servers.push(ServerOptions::new("of", "https://alemi.dev/mcbots/of"));
		Self { servers }
	}
}

impl ServerOptions {
	fn new(title:&str, url:&str) -> Self {
		Self {
			title: title.to_string(),
			url: url.to_string(),
			player_count: PlayerCountData::new(60),
			tps: TpsData::new(15),
			chat: ChatData::new(30),
			sync_time: false,
		}
	}

	fn display(&mut self, ui:&mut eframe::egui::Ui) {
		ui.vertical(|ui| {
			ui.horizontal(|ui| {
				ui.heading(self.title.as_str());
				ui.checkbox(&mut self.sync_time, "Lock X to now");
			});
			let mut p = Plot::new(format!("plot-{}", self.title)).x_axis_formatter(|x, _range| {
				format!(
					"{}",
					DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(x as i64, 0), Utc)
						.format("%Y/%m/%d %H:%M:%S")
				)
			}).center_x_axis(false).height(260.0); // TODO make it fucking reactive! It fills the whole screen with 1 plot no matter what I do...

			if self.sync_time {
				p = p.include_x(Utc::now().timestamp() as f64);
			}

			p.show(ui, |plot_ui| {
				plot_ui.line(
					Line::new(Values::from_values(self.player_count.ds.view())).name("Player Count"),
				);
				plot_ui.line(Line::new(Values::from_values(self.tps.ds.view())).name("TPS over 15s"));
				plot_ui.line(
					Line::new(Values::from_values(self.chat.ds.view()))
						.name("Chat messages per minute"),
				);
			});
		});
	}
}

impl eframe::App for App {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		for server in &mut self.servers {
			server.tps.load_remote(server.url.as_str(), ctx.clone());
			server.player_count.load_remote(server.url.as_str(), ctx.clone());
			server.chat.load_remote(server.url.as_str(), ctx.clone());
		}
		egui::TopBottomPanel::top("??? wtf").show(ctx, |ui| {
			ui.horizontal(|ui| {
				egui::widgets::global_dark_light_mode_switch(ui);
				ui.heading("nnbot dashboard");
				if ui.button("save").clicked() {
					for server in &self.servers {
						native_save(format!("{}-tps.json", server.title).as_str(), server.tps.ds.serialize()).unwrap();
						native_save(format!("{}-chat.json", server.title).as_str(), server.chat.ds.serialize()).unwrap();
						native_save(format!("{}-players.json", server.title).as_str(), server.player_count.ds.serialize()).unwrap();
					}
				}
				if ui.button("load").clicked() {
					for server in &mut self.servers {
						server.tps.load_local(format!("{}-tps.json", server.title).as_str(), ctx.clone());
						server.chat.load_local(format!("{}-chat.json", server.title).as_str(), ctx.clone());
						server.player_count.load_local(format!("{}-players.json", server.title).as_str(), ctx.clone());
					}
				}
				ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
					if ui.button("x").clicked() {
						frame.quit();
					}
				});
			});
		});
		egui::CentralPanel::default().show(ctx, |ui| {
			ui.group(|v_ui| {
				self.servers[0].display(v_ui);
			});
			ui.group(|v_ui| {
				self.servers[1].display(v_ui);
			});
			ui.group(|v_ui| {
				self.servers[2].display(v_ui);
			});
		});
		ctx.request_repaint(); // TODO super jank way to sorta keep drawing
	}
}
