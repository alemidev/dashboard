use chrono::{Local, Utc};
use eframe::{egui::{
	plot::{Corner, GridMark, Legend, Line, Plot},
	DragValue, Layout, Ui, Slider, TextEdit,
}, emath::Vec2};

use crate::app::{
	data::source::{Panel, Source},
	util::timestamp_to_str,
};

pub fn panel_edit_inline_ui(ui: &mut Ui, panel: &mut Panel) {
	TextEdit::singleline(&mut panel.name)
		.hint_text("name")
		.desired_width(100.0)
		.show(ui);
}

pub fn panel_title_ui(ui: &mut Ui, panel: &mut Panel, extra: bool) {
	ui.horizontal(|ui| {
		ui.heading(panel.name.as_str());
		ui.with_layout(Layout::right_to_left(), |ui| {
			ui.horizontal(|ui| {
				ui.toggle_value(&mut panel.view_scroll, "🔒");
				ui.separator();
				if panel.limit {
					ui.label("min"); // TODO makes no sense if it's not a timeserie
					ui.add(
						DragValue::new(&mut panel.view_size)
							.speed(10)
							.clamp_range(0..=2147483647i32),
					);
				}
				ui.toggle_value(&mut panel.limit, "limit");
				ui.separator();
				if panel.shift {
					ui.label("min");
					ui.add(
						DragValue::new(&mut panel.view_offset)
							.speed(10)
							.clamp_range(0..=2147483647i32),
					);
				}
				ui.toggle_value(&mut panel.shift, "offset");
				ui.separator();
				if panel.reduce {
					ui.label("x");
					ui.add(
						DragValue::new(&mut panel.view_chunks)
							.speed(1)
							.clamp_range(1..=1000), // TODO allow to average larger spans maybe?
					);
				}
				ui.toggle_value(&mut panel.reduce, "reduce");
				if extra {
					ui.separator();
					ui.checkbox(&mut panel.timeserie, "timeserie");
					ui.separator();
					ui.add(Slider::new(&mut panel.height, 0..=500).text("height"));
				}
			});
		});
	});
}

pub fn panel_body_ui(ui: &mut Ui, panel: &mut Panel, sources: &Vec<Source>) {
	let mut p = Plot::new(format!("plot-{}", panel.name))
		.height(panel.height as f32)
		.allow_scroll(false)
		.legend(Legend::default().position(Corner::LeftTop));

	if panel.limit {
		p = p.set_margin_fraction(Vec2 { x: 0.0, y: 0.1 });
	}

	if panel.view_scroll {
		let _now = (Utc::now().timestamp() as f64) - (60.0 * panel.view_offset as f64);
		p = p.include_x(_now);
		if panel.limit {
			p = p
				.include_x(_now + (panel.view_size as f64 * 3.0))
				.include_x(_now - (panel.view_size as f64 * 60.0)); // ??? TODO
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
		for source in sources {
			if source.panel_id == panel.id {
				let _now = Utc::now().timestamp() as f64;
				let _off = (panel.view_offset as f64) * 60.0; // TODO multiplying x60 makes sense only for timeseries
				let _size = (panel.view_size as f64) * 60.0; // TODO multiplying x60 makes sense only for timeseries
				let min_x = if panel.limit { Some(_now - _size - _off) } else { None };
				let max_x = if panel.shift { Some(_now - _off) } else { None };
				let chunk_size = if panel.reduce { Some(panel.view_chunks) } else { None };
				// let chunks = None;
				let line = Line::new(source.values(min_x, max_x, chunk_size)).name(source.name.as_str());
				plot_ui.line(line.color(source.color));
			}
		}
	});
}
