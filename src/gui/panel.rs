use chrono::{Local, Utc};
use eframe::{egui::{
	plot::{Corner, GridMark, Legend, Line, Plot},
	Ui, ScrollArea, collapsing_header::CollapsingState, Context, Layout, Slider, DragValue,
}, emath::Vec2};

use crate::util::{timestamp_to_str, unpack_color};
use crate::gui::App;
use crate::data::entities;

use super::scaffold::EditingModel;

pub fn main_content(app: &mut App, ctx: &Context, ui: &mut Ui) {
	let panel_metric = app.view.panel_metric.borrow();
	let metrics = app.view.metrics.borrow();
	let points = app.view.points.borrow();
	ScrollArea::vertical().show(ui, |ui| {
		ui.separator();
		if app.edit {
			for mut panel in app.panels.iter_mut() {
				CollapsingState::load_with_default_open(
					ctx,
					ui.make_persistent_id(format!("panel-{}-compressable", panel.id)),
					true,
				)
				.show_header(ui, |ui| {
					panel_title_ui_edit(ui, &mut panel, &mut app.editing, &metrics, &panel_metric);
				})
				.body(|ui| panel_body_ui(ui, panel, &metrics, &points, &panel_metric));
				ui.separator();
			}
		} else {
			for panel in app.view.panels.borrow().iter() {
				CollapsingState::load_with_default_open(
					ctx,
					ui.make_persistent_id(format!("panel-{}-compressable", panel.id)),
					true,
				)
				.show_header(ui, |ui| {
					panel_title_ui(ui, &panel, &mut app.editing, &metrics, &panel_metric);
				})
				.body(|ui| panel_body_ui(ui, panel, &metrics, &points, &panel_metric));
				ui.separator();
			}
		}
	});
}

pub fn panel_title_ui(
	ui: &mut Ui,
	panel: &entities::panels::Model,
	editing: &mut Vec<EditingModel>,
	metrics: &Vec<entities::metrics::Model>,
	panel_metric: &Vec<entities::panel_metric::Model>,
) { // TODO make edit UI in separate func
	ui.horizontal(|ui| {
		ui.separator();
		if ui.small_button("#").clicked() {
			// TODO don't add duplicates
			editing.push(
				EditingModel::make_edit_panel(panel.clone(), metrics, panel_metric)
			);
		}
		ui.separator();
		ui.heading(panel.name.as_str());
	});
}

pub fn panel_title_ui_edit(
	ui: &mut Ui,
	panel: &mut entities::panels::Model,
	editing: &mut Vec<EditingModel>,
	metrics: &Vec<entities::metrics::Model>,
	panel_metric: &Vec<entities::panel_metric::Model>,
) { // TODO make edit UI in separate func
	ui.horizontal(|ui| {
		ui.separator();
		if ui.small_button("#").clicked() {
			// TODO don't add duplicates
			editing.push(
				EditingModel::make_edit_panel(panel.clone(), metrics, panel_metric)
			);
		}
		ui.separator();
		ui.heading(panel.name.as_str());
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
				ui.separator();
				ui.add(Slider::new(&mut panel.height, 0..=500).text("height"));
			});
		});
	});
}

pub fn panel_body_ui(
	ui: &mut Ui,
	panel: &entities::panels::Model,
	metrics: &Vec<entities::metrics::Model>,
	points: &Vec<entities::points::Model>,
	panel_metric: &Vec<entities::panel_metric::Model>,
) {
	let mut p = Plot::new(format!("plot-{}", panel.name))
		.height(panel.height as f32)
		.allow_scroll(false)
		.legend(Legend::default().position(Corner::LeftTop));

	if panel.view_scroll {
		p = p.set_margin_fraction(Vec2 { x: 0.0, y: 0.1 });
	}


	if panel.view_scroll {
		let now = (Utc::now().timestamp() as f64) - (60.0 * panel.view_offset as f64);
		p = p.include_x(now)
				.include_x(now + (panel.view_size as f64 * 3.0))
				.include_x(now - (panel.view_size as f64 * 60.0)); // ??? TODO
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

	let mut lines : Vec<Line> = Vec::new();
	let now = Utc::now().timestamp() as f64;
	let off = (panel.view_offset as f64) * 60.0; // TODO multiplying x60 makes sense only for timeseries
	let size = (panel.view_size as f64) * 60.0; // TODO multiplying x60 makes sense only for timeseries
	let min_x = now - size - off;
	let max_x = now - off;
	let chunk_size = if panel.reduce_view { Some(panel.view_chunks) } else { None };
	let metric_ids : Vec<i64> = panel_metric.iter().filter(|x| x.panel_id == panel.id).map(|x| x.metric_id).collect();
	for metric in metrics {
		if metric_ids.contains(&metric.id) {
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
