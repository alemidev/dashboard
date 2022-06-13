use eframe::egui;
use eframe::egui::Ui;

use crate::app::data::source::{Panel, Source};

pub fn source_edit_inline_ui(ui: &mut Ui, source: &mut Source, panels: &Vec<Panel>) {
	eframe::egui::TextEdit::singleline(&mut source.name)
		.hint_text("name")
		.desired_width(50.0)
		.show(ui);
	eframe::egui::TextEdit::singleline(&mut source.url)
		.hint_text("url")
		.desired_width(160.0)
		.show(ui);
	eframe::egui::TextEdit::singleline(&mut source.query_x)
		.hint_text("x")
		.desired_width(30.0)
		.show(ui);
	eframe::egui::TextEdit::singleline(&mut source.query_y)
		.hint_text("y")
		.desired_width(30.0)
		.show(ui);
	egui::ComboBox::from_id_source("panel")
		.selected_text(format!("panel [{}]", source.panel_id))
		.width(70.0)
		.show_ui(ui, |ui| {
			for p in panels {
				ui.selectable_value(&mut source.panel_id, p.id, p.name.as_str());
			}
		});
	ui.checkbox(&mut source.visible, "visible");
	ui.add(egui::Slider::new(&mut source.interval, 1..=60));
	ui.color_edit_button_srgba(&mut source.color);
}

pub fn source_ui(ui: &mut Ui, source: &mut Source, panels: &Vec<Panel>) {
	ui.group(|ui| {
		ui.horizontal(|ui| {
			ui.checkbox(&mut source.visible, "");
			eframe::egui::TextEdit::singleline(&mut source.name)
				.hint_text("name")
				.desired_width(80.0)
				.show(ui);
			eframe::egui::TextEdit::singleline(&mut source.url)
				.hint_text("url")
				.desired_width(300.0)
				.show(ui);
		});
		ui.horizontal(|ui| {
			ui.add(egui::Slider::new(&mut source.interval, 1..=60));
			eframe::egui::TextEdit::singleline(&mut source.query_x)
				.hint_text("x")
				.desired_width(50.0)
				.show(ui);
			eframe::egui::TextEdit::singleline(&mut source.query_y)
				.hint_text("y")
				.desired_width(50.0)
				.show(ui);
			egui::ComboBox::from_id_source(format!("panel-{}", source.id))
				.selected_text(format!("panel [{}]", source.panel_id))
				.width(70.0)
				.show_ui(ui, |ui| {
					for p in panels {
						ui.selectable_value(&mut source.panel_id, p.id, p.name.as_str());
					}
				});
			ui.color_edit_button_srgba(&mut source.color);
		});
	});
}
