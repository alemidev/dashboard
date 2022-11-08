use eframe::{egui::{Ui, Sense, color_picker::show_color_at, TextEdit}, epaint::Color32};

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

pub fn metric_line_ui(ui: &mut Ui, metric: &entities::metrics::Model) {
	let mut name = metric.name.clone();
	let mut query = metric.query.clone();
	ui.horizontal(|ui| {
		// ui.color_edit_button_srgba(&mut unpack_color(metric.color));
		color_square(ui, unpack_color(metric.color));
		let unit = (ui.available_width() - 65.0) / 5.0;
		TextEdit::singleline(&mut name)
			.desired_width(unit * 2.0)
			.interactive(false)
			.hint_text("name")
			.show(ui);
		ui.separator();
		TextEdit::singleline(&mut query)
			.desired_width(unit * 3.0)
			.interactive(false)
			.hint_text("query")
			.show(ui);
	});
}
