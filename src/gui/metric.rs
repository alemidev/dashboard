use eframe::{egui::{Ui, Layout, Sense, color_picker::show_color_at, TextEdit}, emath::Align, epaint::Color32};

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

pub fn _metric_display_ui(ui: &mut Ui, metric: &entities::metrics::Model, _width: f32) {
	ui.horizontal(|ui| {
		color_square(ui, unpack_color(metric.color));
		ui.label(&metric.name);
		ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
			ui.horizontal(|ui| {
				ui.label("panel: ???");
				ui.label(format!("y: {}", metric.query_y));
				// if let Some(query_x) = metric.query_x {
				// 	ui.label(format!("x: {}", query_x));
				// }
			})
		});
	});
}

pub fn metric_edit_ui(ui: &mut Ui, metric: &entities::metrics::Model) {
	let mut name = metric.name.clone();
	let mut query_x = metric.query_x.clone();
	let mut query_y = metric.query_y.clone();
	ui.horizontal(|ui| {
		// ui.color_edit_button_srgba(&mut unpack_color(metric.color));
		color_square(ui, unpack_color(metric.color));
		let available = ui.available_width() - 79.0;
		TextEdit::singleline(&mut name)
			.desired_width(available / 2.0)
			.interactive(false)
			.hint_text("name")
			.show(ui);
		ui.separator();
		TextEdit::singleline(&mut query_x)
			.desired_width(available / 4.0)
			.interactive(false)
			.hint_text("x")
			.show(ui);
		TextEdit::singleline(&mut query_y)
			.desired_width(available / 4.0)
			.interactive(false)
			.hint_text("y")
			.show(ui);
	});
}
