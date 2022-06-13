use chrono::{Local, Utc};
use eframe::egui::{
	plot::{Corner, GridMark, Legend, Line, Plot},
	DragValue, Layout, Slider, Ui,
};

use crate::app::{
	data::source::{Panel, Source},
	util::timestamp_to_str,
};

pub fn panel_edit_inline_ui(ui: &mut Ui, panel: &mut Panel) {
	eframe::egui::TextEdit::singleline(&mut panel.name)
		.hint_text("name")
		.desired_width(50.0)
		.show(ui);
}

pub fn panel_title_ui(ui: &mut Ui, panel: &mut Panel) {
	ui.horizontal(|ui| {
		ui.heading(panel.name.as_str());
		ui.with_layout(Layout::right_to_left(), |ui| {
			ui.horizontal(|ui| {
				ui.toggle_value(&mut panel.view_scroll, " • ");
				ui.separator();
				ui.label("m");
				ui.add(
					DragValue::new(&mut panel.view_size)
						.speed(10)
						.clamp_range(0..=2147483647i32),
				);
				ui.checkbox(&mut panel.limit, "limit");
				ui.separator();
				ui.checkbox(&mut panel.timeserie, "timeserie");
				ui.separator();
				ui.add(Slider::new(&mut panel.height, 0..=500).text("height"));
				ui.separator();
			});
		});
	});
}

pub fn panel_body_ui(ui: &mut Ui, panel: &mut Panel, sources: &Vec<Source>) {
	let mut p = Plot::new(format!("plot-{}", panel.name))
		.height(panel.height as f32)
		.allow_scroll(false)
		.legend(Legend::default().position(Corner::LeftTop));

	if panel.view_scroll {
		p = p.include_x(Utc::now().timestamp() as f64);
		if panel.limit {
			p = p
				.set_margin_fraction(eframe::emath::Vec2 { x: 0.0, y: 0.1 })
				.include_x((Utc::now().timestamp() + (panel.view_size as i64 * 3)) as f64);
		}
		if panel.limit {
			p = p.include_x((Utc::now().timestamp() - (panel.view_size as i64 * 60)) as f64);
		}
	}

	if panel.timeserie {
		p = p
			.x_axis_formatter(|x, _range| timestamp_to_str(x as i64, true, false))
			.label_formatter(|name, value| {
				if !name.is_empty() {
					return format!(
						"{}\nx = {}\ny = {:.1}",
						name,
						timestamp_to_str(value.x as i64, false, true),
						value.y
					);
				} else {
					return format!(
						"x = {}\ny = {:.1}",
						timestamp_to_str(value.x as i64, false, true),
						value.y
					);
				}
			})
			.x_grid_spacer(|grid| {
				let offset = Local::now().offset().local_minus_utc() as i64;
				let (start, end) = grid.bounds;
				let mut counter = (start as i64) - ((start as i64) % 3600);
				let mut out: Vec<GridMark> = Vec::new();
				loop {
					counter += 3600;
					if counter > end as i64 {
						break;
					}
					if (counter + offset) % 86400 == 0 {
						out.push(GridMark {
							value: counter as f64,
							step_size: 86400 as f64,
						})
					} else if counter % 3600 == 0 {
						out.push(GridMark {
							value: counter as f64,
							step_size: 3600 as f64,
						});
					}
				}
				return out;
			});
	}

	p.show(ui, |plot_ui| {
		for source in &*sources {
			if source.visible && source.panel_id == panel.id {
				let line = if panel.limit {
					Line::new(source.values_filter(
						(Utc::now().timestamp() - (panel.view_size as i64 * 60)) as f64,
					))
					.name(source.name.as_str())
				} else {
					Line::new(source.values()).name(source.name.as_str())
				};
				plot_ui.line(line.color(source.color));
			}
		}
	});
}
