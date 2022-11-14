pub mod panel;
pub mod source;
pub mod metric;

mod scaffold;

use chrono::Utc;
use eframe::egui::{CentralPanel, Context, SidePanel, TopBottomPanel, Window};
use tokio::sync::{watch, mpsc};
use tracing::error;

use crate::{data::entities, worker::{visualizer::AppStateView, BackgroundAction}};
use panel::main_content;
use scaffold::{
	// confirmation_popup_delete_metric, confirmation_popup_delete_source, footer,
	header,
};
use source::source_panel_ui;

use self::scaffold::{footer, EditingModel, popup_edit_ui};

pub struct App {
	view: AppStateView,
	db_uri: String,
	db_uri_tx: mpsc::Sender<String>,
	last_db_uri: String,
	interval: i64,
	last_redraw: i64,

	panels: Vec<entities::panels::Model>,
	width_tx: watch::Sender<i64>,
	logger_view: watch::Receiver<Vec<String>>,

	// buffer_panel: entities::panels::Model,
	buffer_source: entities::sources::Model,
	buffer_metric: entities::metrics::Model,

	edit: bool,
	editing: Vec<EditingModel>,
	sidebar: bool,
	_padding: bool,
	// windows: Vec<Window<'open>>,
}

impl App {
	pub fn new(
		_cc: &eframe::CreationContext,
		initial_uri: Option<String>,
		db_uri_tx: mpsc::Sender<String>,
		interval: i64,
		view: AppStateView,
		width_tx: watch::Sender<i64>,
		logger_view: watch::Receiver<Vec<String>>,
	) -> Self {
		let panels = view.panels.borrow().clone();
		if let Some(initial_uri) = &initial_uri {
			if let Err(e) = db_uri_tx.blocking_send(initial_uri.clone()) {
				error!(target: "app", "Could not send initial uri: {:?}", e);
			}
		}
		Self {
			db_uri_tx, interval, panels, width_tx, view, logger_view,
			last_db_uri: "[disconnected]".into(),
			db_uri: initial_uri.unwrap_or("".into()),
			buffer_source: entities::sources::Model::default(),
			buffer_metric: entities::metrics::Model::default(),
			last_redraw: 0,
			edit: false,
			editing: vec![],
			sidebar: true,
			_padding: false,
			// windows: vec![],
		}
	}

	pub fn save_all_panels(&self) { // TODO can probably remove this and invoke op() directly
		let msg = BackgroundAction::UpdateAllPanels { panels: self.panels.clone() };
		self.op(msg);
	}

	pub fn refresh_data(&self) {
		let flush_clone = self.view.flush.clone();
		std::thread::spawn(move || {
			if let Err(e) = flush_clone.blocking_send(()) {
				error!(target: "app-background", "Could not request flush: {:?}", e);
			}
		});
	}

	pub fn op(&self, op: BackgroundAction) {
		let op_clone = self.view.op.clone();
		std::thread::spawn(move || {
			if let Err(e) = op_clone.blocking_send(op) {
				error!(target: "app-background", "Could not send operation: {:?}", e);
			}
		});
	}

	fn update_db_uri(&self) {
		let db_uri_clone = self.db_uri_tx.clone();
		let db_uri_str = self.db_uri.clone();
		let flush_clone = self.view.flush.clone();
		std::thread::spawn(move || {
			if let Err(e) = db_uri_clone.blocking_send(db_uri_str) {
				error!(target: "app-background", "Could not send new db uri : {:?}", e);
			}
			if let Err(e) = flush_clone.blocking_send(()) {
				error!(target: "app-background", "Could not request data flush : {:?}", e);
			}
		});
	}
}

impl eframe::App for App {
	fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
		TopBottomPanel::top("header").show(ctx, |ui| {
			header(self, ui, frame);
		});

		TopBottomPanel::bottom("footer").show(ctx, |ui| {
			footer(ctx, ui, self.logger_view.clone(), self.last_db_uri.clone(), self.view.points.borrow().len());
		});

		for m in self.editing.iter_mut() {
			Window::new(m.id_repr())
				.default_width(150.0)
				.show(ctx, |ui| popup_edit_ui(ui, m, &self.view.sources.borrow(), &self.view.metrics.borrow()));
		}

		if self.sidebar {
			SidePanel::left("sources-bar")
				.width_range(280.0..=800.0)
				.default_width(if self.edit { 450.0 } else { 330.0 })
				.show(ctx, |ui| source_panel_ui(self, ui));
		}

		CentralPanel::default().show(ctx, |ui| {
			main_content(self, ctx, ui);
		});

		if let Some(viewsize) =
			self.panels
				.iter()
				.chain(self.view.panels.borrow().iter())
				.map(|p| p.view_size + p.view_offset)
				.max()
		{
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
