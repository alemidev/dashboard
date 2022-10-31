pub mod panel;
pub mod source;
pub mod metric;

mod scaffold;

use eframe::egui::{CentralPanel, Context, SidePanel, TopBottomPanel};
use tokio::sync::watch;

use crate::data::entities;
use panel::main_content;
use scaffold::{
	// confirmation_popup_delete_metric, confirmation_popup_delete_source, footer,
	header,
};
use source::source_panel;

pub struct App {
	panels_rx: watch::Receiver<Vec<entities::panels::Model>>,
	panels: Vec<entities::panels::Model>,
	view_tx: watch::Sender<i64>,

	sources: watch::Receiver<Vec<entities::sources::Model>>,
	metrics: watch::Receiver<Vec<entities::metrics::Model>>,
	points: watch::Receiver<Vec<entities::points::Model>>,

	buffer_panel: entities::panels::Model,
	buffer_source: entities::sources::Model,
	buffer_metric: entities::metrics::Model,

	edit: bool,
	sidebar: bool,
	padding: bool,
}

impl App {
	pub fn new(
		_cc: &eframe::CreationContext,
		panels_rx: watch::Receiver<Vec<entities::panels::Model>>,
		sources: watch::Receiver<Vec<entities::sources::Model>>,
		metrics: watch::Receiver<Vec<entities::metrics::Model>>,
		points: watch::Receiver<Vec<entities::points::Model>>,
		view_tx: watch::Sender<i64>,
	) -> Self {
		let panels = panels_rx.borrow().clone();
		Self {
			panels_rx, panels, view_tx,
			sources, metrics, points,
			buffer_panel: entities::panels::Model::default(),
			buffer_source: entities::sources::Model::default(),
			buffer_metric: entities::metrics::Model::default(),
			edit: false,
			sidebar: true,
			padding: false,
		}
	}
}

impl eframe::App for App {
	fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
		TopBottomPanel::top("header").show(ctx, |ui| {
			header(self, ui, frame);
		});

		TopBottomPanel::bottom("footer").show(ctx, |_ui| {
			// footer(self.data.clone(), ctx, ui);
		});

		// if let Some(index) = self.deleting_metric {
		// 	Window::new(format!("Delete Metric #{}?", index))
		// 		.show(ctx, |ui| confirmation_popup_delete_metric(self, ui, index));
		// }
		// if let Some(index) = self.deleting_source {
		// 	Window::new(format!("Delete Source #{}?", index))
		// 		.show(ctx, |ui| confirmation_popup_delete_source(self, ui, index));
		// }

		if self.sidebar {
			SidePanel::left("sources-bar")
				.width_range(if self.edit { 400.0..=1000.0 } else { 280.0..=680.0 })
				.default_width(if self.edit { 450.0 } else { 330.0 })
				.show(ctx, |ui| source_panel(self, ui));
		}

		CentralPanel::default().show(ctx, |ui| {
			main_content(self, ctx, ui);
		});

		if let Some(viewsize) = self.panels.iter().map(|p| p.view_size).max() {
			self.view_tx.send(viewsize as i64).unwrap();
		}
	}
}
