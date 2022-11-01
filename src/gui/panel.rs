use chrono::{Local, Utc};
use eframe::{egui::{
	plot::{Corner, GridMark, Legend, Line, Plot},
	Ui, ScrollArea, collapsing_header::CollapsingState, Context, Layout, Slider, DragValue,
}, emath::Vec2};

use crate::util::{timestamp_to_str, unpack_color};
use crate::gui::App;
use crate::data::entities;

pub fn main_content(app: &mut App, ctx: &Context, ui: &mut Ui) {
	let mut _to_swap: Option<usize> = None;
	let mut _to_delete: Option<usize> = None;
	ScrollArea::vertical().show(ui, |ui| {
		let panels = &mut app.panels;
		let _panels_count = panels.len();
		let metrics = app.view.metrics.borrow();
		for (index, panel) in panels.iter_mut().enumerate() {
			if index > 0 {
				ui.separator(); // only show this if there is at least one panel
			}
			CollapsingState::load_with_default_open(
				ctx,
				ui.make_persistent_id(format!("panel-{}-compressable", panel.id)),
				true,
			)
			.show_header(ui, |ui| {
				// if ui.small_button(" + ").clicked() {
				// 	if index > 0 {
				// 		to_swap = Some(index); // TODO kinda jank but is there a better way?
				// 	}
				// }
				// if ui.small_button(" − ").clicked() {
				// 	if index < panels_count - 1 {
				// 		to_swap = Some(index + 1); // TODO kinda jank but is there a better way?
				// 	}
				// }
				// if ui.small_button(" × ").clicked() {
				// 	to_delete = Some(index); // TODO kinda jank but is there a better way?
				// }
				// ui.separator();
				panel_title_ui(ui, panel, app.edit);
			})
			.body(|ui| panel_body_ui(ui, panel, &metrics, &app.view.points.borrow()));
		}
	});
}

pub fn panel_edit_inline_ui(_ui: &mut Ui, _panel: &entities::panels::Model) {
	// TextEdit::singleline(&mut panel.name)
	// 	.hint_text("name")
	// 	.desired_width(100.0)
	// 	.show(ui);
}

pub fn panel_title_ui(ui: &mut Ui, panel: &mut entities::panels::Model, _edit: bool) { // TODO make edit UI in separate func
	ui.horizontal(|ui| {
		ui.heading(panel.name.as_str());
		ui.separator();
		ui.add(Slider::new(&mut panel.height, 0..=500).text("height"));
		//ui.separator();
		//ui.checkbox(&mut panel.timeserie, "timeserie");
		ui.with_layout(Layout::right_to_left(eframe::emath::Align::Min), |ui| {
			ui.horizontal(|ui| {
				ui.toggle_value(&mut panel.view_scroll, "🔒");
				ui.separator();
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
				ui.separator();
				if panel.reduce_view {
					ui.add(
						DragValue::new(&mut panel.view_chunks)
							.speed(1)
							.prefix("x")
							.clamp_range(1..=1000), // TODO allow to average larger spans maybe?
					);
					ui.toggle_value(&mut panel.average_view, "avg");
				}
				ui.toggle_value(&mut panel.reduce_view, "reduce");
			});
		});
	});
}

pub fn panel_body_ui(ui: &mut Ui, panel: &entities::panels::Model, metrics: &Vec<entities::metrics::Model>, points: &Vec<entities::points::Model>) {
	let mut p = Plot::new(format!("plot-{}", panel.name))
		.height(panel.height as f32)
		.allow_scroll(false)
		.legend(Legend::default().position(Corner::LeftTop));

	if panel.limit_view {
		p = p.set_margin_fraction(Vec2 { x: 0.0, y: 0.1 });
	}


	if panel.timeserie {
		if panel.view_scroll {
			let _now = (Utc::now().timestamp() as f64) - (60.0 * panel.view_offset as f64);
			p = p.include_x(_now);
			if panel.limit_view {
				p = p
					.include_x(_now + (panel.view_size as f64 * 3.0))
					.include_x(_now - (panel.view_size as f64 * 60.0)); // ??? TODO
			}
		}
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

	let mut lines : Vec<Line> = Vec::new();
	let now = Utc::now().timestamp() as f64;
	let off = (panel.view_offset as f64) * 60.0; // TODO multiplying x60 makes sense only for timeseries
	let size = (panel.view_size as f64) * 60.0; // TODO multiplying x60 makes sense only for timeseries
	let min_x = now - size - off;
	let max_x = now - off;
	let chunk_size = if panel.reduce_view { Some(panel.view_chunks) } else { None };
	for metric in metrics {
		if metric.panel_id == panel.id {
			// let values = metric.values(min_x, max_x, chunk_size, panel.average_view); 
			let mut values : Vec<[f64;2]> = points
				.iter()
				.filter(|v| v.metric_id == metric.id)
				.filter(|v| v.x > min_x as f64)
				.filter(|v| v.x < max_x as f64)
				.map(|v| [v.x, v.y])
				.collect();
			if let Some(chunk_size) = chunk_size { // TODO make this less of a mess
				let iter = values.chunks(chunk_size as usize);
				values = iter.map(|x| 
					if panel.average_view { avg_value(x) } else { 
						if x.len() > 0 { x[x.len()-1] } else { [0.0, 0.0 ]}
					}).collect();
			}
			// if !panel.timeserie && panel.view_scroll && values.len() > 0 {
			// 	let l = values.len() - 1;
			// 	p = p.include_x(values[0].x)
			// 		.include_x(values[l].x)
			// 		.include_y(values[0].y)
			// 		.include_y(values[l].y);
			// }
			lines.push(
				Line::new(values)
					.name(metric.name.as_str())
					.color(unpack_color(metric.color))
			);
		}
	}

	p.show(ui, |plot_ui| {
		for line in lines {
			plot_ui.line(line);
		}
	});
}

fn avg_value(values: &[[f64;2]]) -> [f64;2] {
	let mut x = 0.0;
	let mut y = 0.0;
	for v in values {
		x += v[0];
		y += v[1];
	}
	return [
		x / values.len() as f64,
		y / values.len() as f64,
	];
}
