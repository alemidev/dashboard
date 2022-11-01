pub mod panel;
pub mod source;
pub mod metric;

mod scaffold;

use chrono::Utc;
use eframe::egui::{CentralPanel, Context, SidePanel, TopBottomPanel};
use tokio::sync::watch;

use crate::{data::entities, worker::visualizer::AppStateView};
use panel::main_content;
use scaffold::{
	// confirmation_popup_delete_metric, confirmation_popup_delete_source, footer,
	header,
};
use source::source_panel;

use self::scaffold::footer;

pub struct App {
	view: AppStateView,
	db_path: String,
	interval: i64,
	last_redraw: i64,

	panels: Vec<entities::panels::Model>,
	width_tx: watch::Sender<i64>,
	logger_view: watch::Receiver<Vec<String>>,

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
		db_path: String,
		interval: i64,
		view: AppStateView,
		width_tx: watch::Sender<i64>,
		logger_view: watch::Receiver<Vec<String>>,
	) -> Self {
		let panels = view.panels.borrow().clone();
		Self {
			db_path, interval, panels, width_tx, view, logger_view,
			buffer_panel: entities::panels::Model::default(),
			buffer_source: entities::sources::Model::default(),
			buffer_metric: entities::metrics::Model::default(),
			last_redraw: 0,
			edit: false,
			sidebar: true,
			padding: false,
		}
	}

	pub fn save_all_panels(&self) {
		self.view.op.blocking_send(
			crate::worker::visualizer::BackgroundAction::UpdateAllPanels { panels: self.panels.clone() }
		).unwrap();
	}

	pub fn refresh_data(&self) {
		self.view.flush.blocking_send(()).unwrap();
	}
}

impl eframe::App for App {
	fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
		TopBottomPanel::top("header").show(ctx, |ui| {
			header(self, ui, frame);
		});

		TopBottomPanel::bottom("footer").show(ctx, |ui| {
			footer(ctx, ui, self.logger_view.clone(), self.db_path.clone(), self.view.points.borrow().len());
		});

		// if let Some(index) = self.deleting_metric {
		// 	Window::new(format!("Delete Metric #{}?", index))
		// 		.show(ctx, |ui| confirmation_popup_delete_metric(self, ui, index));
		// }
		// if let Some(index) = self.deleting_source {
		// 	Window::new(format!("Delete Source #{}?", index))
		// 		.show(ctx, |ui| confirmation_popup_delete_source(self, ui, index));
		// }

		// for window in self.windows {

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
			self.width_tx.send(viewsize as i64).unwrap();
		}

		if Utc::now().timestamp() > self.last_redraw + self.interval {
			ctx.request_repaint();
			self.last_redraw = Utc::now().timestamp();
		}
	}
}
