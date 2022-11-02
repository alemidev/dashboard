use eframe::{Frame, egui::{collapsing_header::CollapsingState, Context, Ui, Layout, ScrollArea, global_dark_light_mode_switch, TextEdit, Checkbox, Slider}, emath::Align};
use sea_orm::{Set, Unchanged, ActiveValue::NotSet};
use tokio::sync::watch;

use crate::{gui::App, data::entities, util::unpack_color, worker::BackgroundAction};

// TODO make this not super specific!
pub fn _confirmation_popup_delete_metric(_app: &mut App, ui: &mut Ui, _metric_index: usize) {
	ui.heading("Are you sure you want to delete this metric?");
	ui.label("This will remove all its metrics and delete all points from archive. This action CANNOT BE UNDONE!");
	ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
		ui.horizontal(|ui| {
			if ui.button("\n   yes   \n").clicked() {
				// let store = app.data.storage.lock().expect("Storage Mutex poisoned");
				// let mut metrics = app.data.metrics.write().expect("Metrics RwLock poisoned");
				// store.delete_metric(metrics[metric_index].id).expect("Failed deleting metric");
				// store.delete_values(metrics[metric_index].id).expect("Failed deleting values");
				// metrics.remove(metric_index);
				// app.deleting_metric = None;
			}
			if ui.button("\n   no    \n").clicked() {
				// app.deleting_metric = None;
			}
		});
	});
}

// TODO make this not super specific!
pub fn _confirmation_popup_delete_source(_app: &mut App, ui: &mut Ui, _source_index: usize) {
	ui.heading("Are you sure you want to delete this source?");
	ui.label("This will remove all its metrics and delete all points from archive. This action CANNOT BE UNDONE!");
	ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
		ui.horizontal(|ui| {
			if ui.button("\n   yes   \n").clicked() {
				// let store = app.data.storage.lock().expect("Storage Mutex poisoned");
				// let mut sources = app.data.sources.write().expect("sources RwLock poisoned");
				// let mut metrics = app.data.metrics.write().expect("Metrics RwLock poisoned");
				// let mut to_remove = Vec::new();
				// for j in 0..metrics.len() {
				// 	if metrics[j].source_id == app.input_source.id {
				// 		store.delete_values(metrics[j].id).expect("Failed deleting values");
				// 		store.delete_metric(metrics[j].id).expect("Failed deleting Metric");
				// 		to_remove.push(j);
				// 	}
				// }
				// for index in to_remove {
				// 	metrics.remove(index);
				// }
				// store.delete_source(sources[source_index].id).expect("Failed deleting source");
				// sources.remove(source_index);
				// app.deleting_source = None;
			}
			if ui.button("\n   no    \n").clicked() {
				// app.deleting_source = None;
			}
		});
	});
}

pub struct EditingModel {
	pub id: i64,
	m: EditingModelType,
	new: bool,
	valid: bool,
	ready: bool,
}

impl EditingModel {
	pub fn id_repr(&self) -> String {
		let prefix = match self.m {
			EditingModelType::EditingPanel { panel: _ } => "panel",
			EditingModelType::EditingSource { source: _ } => "source",
			EditingModelType::EditingMetric { metric: _ } => "metric",
		};
		format!("edit_{}_{}", prefix, self.id)
	}

	pub fn should_fetch(&self) -> bool {
		return self.ready && self.valid;
	}

	pub fn modifying(&self) -> bool {
		return !self.ready;
	}

	pub fn to_msg(&self) -> BackgroundAction {
		match &self.m {
			EditingModelType::EditingPanel { panel } =>
				BackgroundAction::UpdatePanel {
					panel: entities::panels::ActiveModel {
						id: if self.new { NotSet } else { Unchanged(panel.id) },
						name: Set(panel.name.clone()),
						view_scroll: Set(panel.view_scroll),
						view_size: Set(panel.view_size),
						timeserie: Set(panel.timeserie),
						height: Set(panel.height),
						limit_view: Set(panel.limit_view),
						position: Set(panel.position),
						reduce_view: Set(panel.reduce_view),
						view_chunks: Set(panel.view_chunks),
						shift_view: Set(panel.shift_view),
						view_offset: Set(panel.view_offset),
						average_view: Set(panel.average_view),
					}
				},
			EditingModelType::EditingSource { source } =>
				BackgroundAction::UpdateSource {
					source: entities::sources::ActiveModel {
						id: if self.new { NotSet } else { Unchanged(source.id) },
						name: Set(source.name.clone()),
						enabled: Set(source.enabled),
						url: Set(source.url.clone()),
						interval: Set(source.interval),
						last_update: Set(source.last_update),
						position: Set(source.position),
					}
				},
			EditingModelType::EditingMetric { metric } =>
				BackgroundAction::UpdateMetric {
					metric: entities::metrics::ActiveModel {
						id: if self.new { NotSet} else { Unchanged(metric.id) },
						name: Set(metric.name.clone()),
						source_id: Set(metric.source_id),
						color: Set(metric.color),
						panel_id: Set(metric.panel_id),
						query_x: Set(metric.query_x.clone()),
						query_y: Set(metric.query_y.clone()),
						position: Set(metric.position),
					}
				},
		}
	}
}

impl From<entities::sources::Model> for EditingModel {
	fn from(s: entities::sources::Model) -> Self {
		EditingModel {
			new: if s.id == 0 { true } else { false },
			id: s.id, m: EditingModelType::EditingSource { source: s }, valid: false, ready: false,
		}
	}
}

impl From<entities::metrics::Model> for EditingModel {
	fn from(m: entities::metrics::Model) -> Self {
		EditingModel {
			new: if m.id == 0 { true } else { false },
			id: m.id, m: EditingModelType::EditingMetric { metric: m }, valid: false, ready: false,
		}
	}
}

impl From<entities::panels::Model> for EditingModel {
	fn from(p: entities::panels::Model) -> Self {
		EditingModel {
			new: if p.id == 0 { true } else { false },
			id: p.id, m: EditingModelType::EditingPanel { panel: p }, valid: false, ready: false,
		}
	}
}

pub enum EditingModelType {
	EditingPanel  { panel : entities::panels::Model  },
	EditingSource { source: entities::sources::Model },
	EditingMetric { metric: entities::metrics::Model },
}

pub fn popup_edit_ui(ui: &mut Ui, model: &mut EditingModel) {
	match &mut model.m {
		EditingModelType::EditingPanel { panel } => {
			ui.heading(format!("Edit panel #{}", panel.id));
			TextEdit::singleline(&mut panel.name)
				.hint_text("name")
				.show(ui);
		},
		EditingModelType::EditingSource { source } => {
			ui.heading(format!("Edit source #{}", source.id));
			ui.horizontal(|ui| {
				ui.add(Checkbox::new(&mut source.enabled, ""));
				TextEdit::singleline(&mut source.name)
					.hint_text("name")
					.show(ui);
			});
			TextEdit::singleline(&mut source.url)
				.hint_text("url")
				.show(ui);
			ui.add(Slider::new(&mut source.interval, 1..=3600).text("interval"));
		},
		EditingModelType::EditingMetric { metric } => {
			ui.heading(format!("Edit metric #{}", metric.id));
			ui.horizontal(|ui| {
				ui.color_edit_button_srgba(&mut unpack_color(metric.color));
				TextEdit::singleline(&mut metric.name)
					.hint_text("name")
					.show(ui);
			});
			TextEdit::singleline(&mut metric.query_x)
				.hint_text("x")
				.show(ui);
			TextEdit::singleline(&mut metric.query_y)
				.hint_text("y")
				.show(ui);
		},
	}
	ui.separator();
	ui.horizontal(|ui| {
		if ui.button("   save   ").clicked() {
			model.valid = true;
			model.ready = true;
		}
		ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
			if ui.button("   close   ").clicked() {
				model.valid = false;
				model.ready = true;
			}
		});
	});
}

pub fn header(app: &mut App, ui: &mut Ui, frame: &mut Frame) {
	ui.horizontal(|ui| {
		global_dark_light_mode_switch(ui);
		ui.heading("dashboard");
		ui.separator();
		ui.checkbox(&mut app.sidebar, "sources");
		ui.separator();
		if ui.button("reset").clicked() {
			app.panels = app.view.panels.borrow().clone();
		}
		ui.separator();
		if ui.button("save").clicked() {
			app.save_all_panels();
		}
		ui.separator();
		if ui.button("refresh").clicked() {
			app.refresh_data();
		}
		ui.separator();
		if ui.button("new panel").clicked() {
			app.editing.push(entities::panels::Model::default().into());
		}
		ui.separator();
		if ui.button("new source").clicked() {
			app.editing.push(entities::sources::Model::default().into());
		}
		ui.separator();
		if ui.button("new metric").clicked() {
			app.editing.push(entities::metrics::Model::default().into());
		}
		// ui.separator();
		// ui.checkbox(&mut app.edit, "edit");
		// if app.edit {
		// 	ui.label("+ panel");
		// 	panel_edit_inline_ui(ui, &mut app.buffer_panel);
		// 	if ui.button("add").clicked() {
		// 		// if let Err(e) = app.data.add_panel(&app.input_panel) {
		// 		// 	error!(target: "ui", "Failed to add panel: {:?}", e);
		// 		// };
		// 	}
		// 	ui.separator();
		// }
		ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
			ui.horizontal(|ui| {
				if ui.small_button("×").clicked() {
					frame.close();
				}
			});
		});
	});
}

pub fn footer(ctx: &Context, ui: &mut Ui, diagnostics: watch::Receiver<Vec<String>>, db_path: String, records: usize) {
	CollapsingState::load_with_default_open(
		ctx,
		ui.make_persistent_id("footer-logs"),
		false,
	)
	.show_header(ui, |ui| {
		ui.horizontal(|ui| {
			ui.separator();
			ui.label(db_path); // TODO maybe calculate it just once?
			ui.separator();
			ui.label(format!("{} records loaded", records)); // TODO put thousands separator
			// ui.label(human_size(
			// 	*data
			// 		.file_size
			// 		.read()
			// 		.expect("Filesize RwLock poisoned"),
			// ));
			ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
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
		ScrollArea::vertical().show(ui, |ui| {
			ui.separator();
			for msg in diagnostics.borrow().iter() {
				ui.label(msg);
			}
			ui.separator();
		});
	});
}
