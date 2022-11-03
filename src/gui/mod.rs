pub mod panel;
pub mod source;
pub mod metric;

mod scaffold;

use chrono::Utc;
use eframe::egui::{CentralPanel, Context, SidePanel, TopBottomPanel, Window};
use tokio::sync::watch;
use tracing::error;

use crate::{data::entities, worker::{visualizer::AppStateView, BackgroundAction}};
use panel::main_content;
use scaffold::{
	// confirmation_popup_delete_metric, confirmation_popup_delete_source, footer,
	header,
};
use source::source_panel;

use self::scaffold::{footer, EditingModel, popup_edit_ui};

pub struct App {
	view: AppStateView,
	db_path: String,
	interval: i64,
	last_redraw: i64,

	panels: Vec<entities::panels::Model>,
	width_tx: watch::Sender<i64>,
	logger_view: watch::Receiver<Vec<String>>,

	// buffer_panel: entities::panels::Model,
	buffer_source: entities::sources::Model,
	// buffer_metric: entities::metrics::Model,

	edit: bool,
	editing: Vec<EditingModel>,
	sidebar: bool,
	padding: bool,
	// windows: Vec<Window<'open>>,
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
			buffer_source: entities::sources::Model::default(),
			last_redraw: 0,
			edit: false,
			editing: vec![],
			sidebar: true,
			padding: false,
			// windows: vec![],
		}
	}

	pub fn save_all_panels(&self) {
		if let Err(e) = self.view.op.blocking_send(
			crate::worker::visualizer::BackgroundAction::UpdateAllPanels { panels: self.panels.clone() }
		) {
			error!(target: "app", "Could not save panels: {:?}", e);
		}
	}

	pub fn refresh_data(&self) {
		if let Err(e) = self.view.flush.blocking_send(()) {
			error!(target: "app", "Could not request flush: {:?}", e);
		}
	}

	pub fn op(&self, op: BackgroundAction) {
		if let Err(e) = self.view.op.blocking_send(op) {
			error!(target: "app", "Could not send operation: {:?}", e);
		}
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

		for m in self.editing.iter_mut() {
			Window::new(m.id_repr())
				.default_width(150.0)
				.show(ctx, |ui| popup_edit_ui(ui, m, &self.view.sources.borrow(), &self.view.metrics.borrow()));
		}

		if self.sidebar {
			SidePanel::left("sources-bar")
				.width_range(if self.edit { 400.0..=1000.0 } else { 280.0..=680.0 })
				.default_width(if self.edit { 450.0 } else { 330.0 })
				.show(ctx, |ui| source_panel(self, ui));
		}

		CentralPanel::default().show(ctx, |ui| {
			main_content(self, ctx, ui);
		});

		if let Some(viewsize) = self.panels.iter().map(|p| p.view_size + p.view_offset).max() {
			if let Err(e) = self.width_tx.send(viewsize as i64) {
				error!(target: "app", "Could not update fetch size : {:?}", e);
			}
		}

		if Utc::now().timestamp() > self.last_redraw + self.interval {
			ctx.request_repaint();
			self.last_redraw = Utc::now().timestamp();
		}

		for m in self.editing.iter() {
			if m.should_fetch() {
				self.op(m.to_msg(self.view.clone())); // TODO cloning is super wasteful
			}
		}

		self.editing.retain(|v| v.modifying());
	}
}
