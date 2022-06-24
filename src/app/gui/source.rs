use eframe::{egui::{Ui, TextEdit, DragValue, Checkbox}};

use crate::app::data::source::Source;

pub fn source_display_ui(ui: &mut Ui, source: &mut Source, _width: f32) {
	ui.horizontal(|ui| {
		ui.add_enabled(false, Checkbox::new(&mut source.enabled, ""));
		ui.add_enabled(false, DragValue::new(&mut source.interval).clamp_range(1..=120));
		ui.heading(&source.name).on_hover_text(&source.url);
	});
}

pub fn source_edit_ui(ui: &mut Ui, source: &mut Source, width: f32) {
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
}
