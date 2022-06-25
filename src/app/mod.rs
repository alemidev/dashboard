pub mod data;
pub mod gui;
pub mod util;
pub mod worker;

use eframe::egui::Window;
use eframe::egui::{CentralPanel, Context, SidePanel, TopBottomPanel};
use std::sync::Arc;

use self::data::source::{Metric, Panel, Source};
use self::data::ApplicationState;
use self::gui::panel::main_content;
use self::gui::scaffold::{
	confirmation_popup_delete_metric, confirmation_popup_delete_source, footer, header,
};
use self::gui::source::source_panel;

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
		TopBottomPanel::top("header").show(ctx, |ui| {
			header(self, ui, frame);
		});

		TopBottomPanel::bottom("footer").show(ctx, |ui| {
			footer(self.data.clone(), ctx, ui);
		});

		if let Some(index) = self.deleting_metric {
			Window::new(format!("Delete Metric #{}", index))
				.show(ctx, |ui| confirmation_popup_delete_metric(self, ui, index));
		}
		if let Some(index) = self.deleting_source {
			Window::new(format!("Delete Source #{}", index))
				.show(ctx, |ui| confirmation_popup_delete_source(self, ui, index));
		}

		if self.sources {
			SidePanel::left("sources-bar")
				.width_range(280.0..=800.0)
				.default_width(330.0)
				.show(ctx, |ui| source_panel(self, ui));
		}

		CentralPanel::default().show(ctx, |ui| {
			main_content(self, ctx, ui);
		});
	}
}
