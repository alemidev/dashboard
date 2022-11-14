use eframe::{Frame, egui::{collapsing_header::CollapsingState, Context, Ui, Layout, ScrollArea, global_dark_light_mode_switch, TextEdit, Checkbox, Slider, ComboBox, DragValue}, emath::Align};
use sea_orm::{Set, Unchanged, ActiveValue::NotSet};
use tokio::sync::watch;

use crate::{gui::App, data::entities, util::{unpack_color, repack_color}, worker::{BackgroundAction, AppStateView}};

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
			EditingModelType::EditingPanel { panel: _, opts: _ } => "panel",
			EditingModelType::EditingSource { source: _ } => "source",
			EditingModelType::EditingMetric { metric: _ } => "metric",
		};
		format!("edit {} #{}", prefix, self.id)
	}

	pub fn should_fetch(&self) -> bool {
		return self.ready && self.valid;
	}

	pub fn modifying(&self) -> bool {
		return !self.ready;
	}

	pub fn make_edit_panel(
		panel: entities::panels::Model,
		metrics: &Vec<entities::metrics::Model>,
		panel_metric: &Vec<entities::panel_metric::Model>
	) -> EditingModel {
		let metric_ids : Vec<i64> = panel_metric.iter().filter(|x| x.panel_id == panel.id).map(|x| x.metric_id).collect();
		let mut opts = vec![false; metrics.len()];
		for i in 0..metrics.len() {
			if metric_ids.contains(&metrics[i].id) {
				opts[i] = true;
			}
		}
		EditingModel {
			id: panel.id,
			new: if panel.id > 0 { false } else { true },
			m: EditingModelType::EditingPanel { panel, opts },
			valid: false,
			ready: false,
		}
	}

	pub fn to_msg(&self, view:AppStateView) -> BackgroundAction {
		match &self.m {
			EditingModelType::EditingPanel { panel, opts: metrics } =>
				BackgroundAction::UpdatePanel {
					panel: entities::panels::ActiveModel {
						id: if self.new { NotSet } else { Unchanged(panel.id) },
						name: Set(panel.name.clone()),
						view_scroll: Set(panel.view_scroll),
						view_size: Set(panel.view_size),
						height: Set(panel.height),
						position: Set(panel.position),
						reduce_view: Set(panel.reduce_view),
						view_chunks: Set(panel.view_chunks),
						view_offset: Set(panel.view_offset),
						average_view: Set(panel.average_view),
					},
					metrics: view.metrics.borrow().iter()
						.enumerate()
						.filter(|(i, _x)| *metrics.get(*i).unwrap_or(&false))
						.map(|(_i, m)| entities::panel_metric::ActiveModel {
							id: NotSet,
							panel_id: Set(panel.id),
							metric_id: Set(m.id),
						})
						.collect(),
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
						query: Set(metric.query.clone()),
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
			id: p.id, m: EditingModelType::EditingPanel { panel: p , opts: vec![] }, valid: false, ready: false,
		}
	}
}

pub enum EditingModelType {
	EditingPanel  { panel : entities::panels::Model, opts: Vec<bool>  },
	EditingSource { source: entities::sources::Model },
	EditingMetric { metric: entities::metrics::Model },
}

pub fn popup_edit_ui(
	ui: &mut Ui,
	model: &mut EditingModel,
	sources: &Vec<entities::sources::Model>,
	metrics: &Vec<entities::metrics::Model>
) {
	match &mut model.m {
		EditingModelType::EditingPanel { panel, opts } => {
			TextEdit::singleline(&mut panel.name)
				.hint_text("name")
				.show(ui);
			ui.horizontal(|ui| {
				ui.label("position");
				ui.add(DragValue::new(&mut panel.position).clamp_range(0..=1000));
			});
			ui.horizontal(|ui| {
				ui.add(
					DragValue::new(&mut panel.view_size)
						.speed(10)
						.suffix(" min")
						.clamp_range(0..=2147483647i32),
				);
				ui.separator();
				ui.add(
					DragValue::new(&mut panel.view_offset)
						.speed(10)
						.suffix(" min")
						.clamp_range(0..=2147483647i32),
				);
			});
			ui.horizontal(|ui| {
				ui.toggle_value(&mut panel.reduce_view, "reduce");
				if panel.reduce_view {
					ui.add(
						DragValue::new(&mut panel.view_chunks)
						.speed(1)
						.suffix("×")
						.clamp_range(1..=1000)
					);
					ui.toggle_value(&mut panel.average_view, "avg");
				}
			});
			ui.label("metrics:");
			ui.group(|ui| {
				for (i, metric) in metrics.iter().enumerate() {
					if i >= opts.len() { // TODO safe but jank: always starts with all off
						opts.push(false);
					}
					ui.checkbox(&mut opts[i], &metric.name);
				}
			});
		},
		EditingModelType::EditingSource { source } => {
			ui.horizontal(|ui| {
				ui.add(Checkbox::new(&mut source.enabled, ""));
				TextEdit::singleline(&mut source.name)
					.hint_text("name")
					.show(ui);
			});
			ui.horizontal(|ui| {
				ui.label("position");
				ui.add(DragValue::new(&mut source.position).clamp_range(0..=1000));
			});
			TextEdit::singleline(&mut source.url)
				.hint_text("url")
				.show(ui);
			ui.add(Slider::new(&mut source.interval, 1..=3600).text("interval"));
		},
		EditingModelType::EditingMetric { metric } => {
			ui.horizontal(|ui| {
				let mut color_buf = unpack_color(metric.color);
				ui.color_edit_button_srgba(&mut color_buf);
				metric.color = repack_color(color_buf);
				TextEdit::singleline(&mut metric.name)
					.hint_text("name")
					.show(ui);
			});
			ui.horizontal(|ui| {
				ui.label("position");
				ui.add(DragValue::new(&mut metric.position).clamp_range(0..=1000));
			});
			ComboBox::from_id_source(format!("source-selector-{}", metric.id))
				.selected_text(format!("source: {:02}", metric.source_id))
				.show_ui(ui, |ui| {
					ui.selectable_value(&mut metric.source_id, -1, "None");
					for s in sources.iter() {
						ui.selectable_value(&mut metric.source_id, s.id, s.name.as_str());
					}
				});
			TextEdit::singleline(&mut metric.query)
				.hint_text("query")
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
		if ui.button("refresh").clicked() {
			app.refresh_data();
		}
		TextEdit::singleline(&mut app.db_uri)
			.hint_text("db uri")
			.show(ui);
		if ui.button("connect").clicked() {
			app.update_db_uri();
			app.last_db_uri = app.db_uri.split("/").last().unwrap_or("").to_string();
		}
		ui.separator();
		let last_edit = app.edit; // replace panels when going into edit mode
		ui.checkbox(&mut app.edit, "edit");
		if app.edit {
			if !last_edit { // TODO kinda cheap fix having it down here
				app.panels = app.view.panels.borrow().clone();
			}
			if ui.button("+").clicked() {
				app.editing.push(entities::panels::Model::default().into());
			}
			if ui.button("reset").clicked() {
				app.panels = app.view.panels.borrow().clone();
			}
			if ui.button("save").clicked() {
				app.save_all_panels();
				app.edit = false;
			}
		}
		ui.separator();
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
