use eframe::{egui::{Ui, TextEdit, DragValue, Checkbox}};

use crate::app::data::source::{Panel, Source, Metric};

use super::metric::{metric_edit_ui, metric_display_ui};

pub fn source_display_ui(ui: &mut Ui, source: &mut Source, metrics: &Vec<Metric>, _width: f32) {
	ui.horizontal(|ui| {
		ui.add_enabled(false, Checkbox::new(&mut source.enabled, ""));
		ui.add_enabled(false, DragValue::new(&mut source.interval).clamp_range(1..=120));
		ui.heading(&source.name).on_hover_text(&source.url);
	});
	for metric in metrics.iter() {
		if metric.source_id == source.id {
			metric_display_ui(ui, metric, ui.available_width());
		}
	}
}

pub fn source_edit_ui(ui: &mut Ui, source: &mut Source, metrics: Option<&mut Vec<Metric>>, panels: &Vec<Panel>, width: f32) {
	ui.horizontal(|ui| {
		let text_width = width - 100.0;
		ui.checkbox(&mut source.enabled, "");
		ui.add(DragValue::new(&mut source.interval).clamp_range(1..=3600));
		TextEdit::singleline(&mut source.name)
			.desired_width(text_width / 4.0)
			.hint_text("name")
			.show(ui);
		TextEdit::singleline(&mut source.url)
			.desired_width(text_width * 3.0 / 4.0)
			.hint_text("url")
			.show(ui);
	});
	if let Some(metrics) = metrics {
		for metric in metrics.iter_mut() {
			if metric.source_id == source.id {
				metric_edit_ui(ui, metric, Some(panels), width - 10.0);
			}
		}
	}
}
