use eframe::egui::{Ui, TextEdit, ComboBox, Slider, DragValue};

use crate::app::data::source::{Panel, Source};

#[allow(dead_code)]
pub fn source_edit_inline_ui(ui: &mut Ui, source: &mut Source, panels: &Vec<Panel>) {
	TextEdit::singleline(&mut source.name)
		.hint_text("name")
		.desired_width(50.0)
		.show(ui);
	TextEdit::singleline(&mut source.url)
		.hint_text("url")
		.desired_width(160.0)
		.show(ui);
	TextEdit::singleline(&mut source.query_x)
		.hint_text("x")
		.desired_width(30.0)
		.show(ui);
	TextEdit::singleline(&mut source.query_y)
		.hint_text("y")
		.desired_width(30.0)
		.show(ui);
	ComboBox::from_id_source("panel")
		.selected_text(format!("panel [{}]", source.panel_id))
		.width(70.0)
		.show_ui(ui, |ui| {
			for p in panels {
				ui.selectable_value(&mut source.panel_id, p.id, p.name.as_str());
			}
		});
	ui.checkbox(&mut source.visible, "visible");
	ui.add(Slider::new(&mut source.interval, 1..=60));
	ui.color_edit_button_srgba(&mut source.color);
}

pub fn source_edit_ui(ui: &mut Ui, source: &mut Source, panels: &Vec<Panel>, width: f32) {
	ui.group(|ui| {
		ui.horizontal(|ui| {
			let text_width = width - 25.0;
			ui.checkbox(&mut source.visible, "");
			TextEdit::singleline(&mut source.name)
				.desired_width(text_width / 4.0)
				.hint_text("name")
				.show(ui);
			TextEdit::singleline(&mut source.url)
				.desired_width(text_width * 3.0 / 4.0)
				.hint_text("url")
				.show(ui);
		});
		ui.horizontal(|ui| {
			let text_width : f32 ;
			if width > 400.0 {
				ui.add(Slider::new(&mut source.interval, 1..=120));
				text_width = width - 330.0
			} else {
				ui.add(DragValue::new(&mut source.interval).clamp_range(1..=120));
				text_width = width - 225.0
			}
			TextEdit::singleline(&mut source.query_x)
				.desired_width(text_width / 2.0)
				.hint_text("x")
				.show(ui);
			TextEdit::singleline(&mut source.query_y)
				.desired_width(text_width / 2.0)
				.hint_text("y")
				.show(ui);
			ComboBox::from_id_source(format!("panel-{}", source.id))
				.width(60.0)
				.selected_text(format!("panel [{}]", source.panel_id))
				.show_ui(ui, |ui| {
					for p in panels {
						ui.selectable_value(&mut source.panel_id, p.id, p.name.as_str());
					}
				});
			ui.color_edit_button_srgba(&mut source.color);
		});
	});
}
