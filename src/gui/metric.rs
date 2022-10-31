use eframe::{egui::{Ui, Layout, Sense, color_picker::show_color_at}, emath::Align, epaint::Color32};

use crate::{data::entities, util::unpack_color};

fn color_square(ui: &mut Ui, color:Color32) {
	let size = ui.spacing().interact_size;
	let (rect, response) = ui.allocate_exact_size(size, Sense::click());
	if ui.is_rect_visible(rect) {
		let visuals = ui.style().interact(&response);
		let rect = rect.expand(visuals.expansion);

		show_color_at(ui.painter(), color, rect);

		let rounding = visuals.rounding.at_most(2.0);
		ui.painter()
				.rect_stroke(rect, rounding, (2.0, visuals.bg_fill)); // fill is intentional, because default style has no border
	}
}

pub fn metric_display_ui(ui: &mut Ui, metric: &entities::metrics::Model, _width: f32) {
	ui.horizontal(|ui| {
		color_square(ui, unpack_color(metric.color));
		ui.label(&metric.name);
		ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
			ui.horizontal(|ui| {
				ui.label(format!("panel: {}", metric.panel_id));
				ui.label(format!("y: {}", metric.query_y));
				// if let Some(query_x) = metric.query_x {
				// 	ui.label(format!("x: {}", query_x));
				// }
			})
		});
	});
}

pub fn metric_edit_ui(ui: &mut Ui, metric: &entities::metrics::Model, panels: Option<&Vec<entities::panels::Model>>, width: f32) {
	let _text_width = width - 195.0;
	ui.horizontal(|ui| {
		ui.color_edit_button_srgba(&mut unpack_color(metric.color));
		// TextEdit::singleline(&mut metric.name)
		// 	.desired_width(text_width / 2.0)
		// 	.hint_text("name")
		// 	.show(ui);
		ui.separator();
		// TextEdit::singleline(&mut metric.query_x.unwrap_or("".into()))
		// 	.desired_width(text_width / 4.0)
		// 	.hint_text("x")
		// 	.show(ui);
		// TextEdit::singleline(&mut metric.query_y)
		// 	.desired_width(text_width / 4.0)
		// 	.hint_text("y")
		// 	.show(ui);
		if let Some(_panels) = panels {
			// ComboBox::from_id_source(format!("panel-selector-{}", metric.id))
			// 	.width(60.0)
			// 	.selected_text(format!("panel: {:02}", metric.panel_id))
			// 	.show_ui(ui, |ui| {
			// 		ui.selectable_value(&mut metric.panel_id, -1, "None");
			// 		for p in panels {
			// 			ui.selectable_value(&mut metric.panel_id, p.id, p.name.as_str());
			// 		}
			// 	});
		}
	});
}
