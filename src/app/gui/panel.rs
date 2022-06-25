use chrono::{Local, Utc};
use eframe::{egui::{
	plot::{Corner, GridMark, Legend, Line, Plot, Values},
	DragValue, Layout, Ui, Slider, TextEdit, ScrollArea, collapsing_header::CollapsingState, Context,
}, emath::Vec2};
use tracing::error;

use crate::app::{
	data::source::{Panel, Metric},
	util::timestamp_to_str, App,
};

pub fn main_content(app: &mut App, ctx: &Context, ui: &mut Ui) {
	let mut to_swap: Option<usize> = None;
	let mut to_delete: Option<usize> = None;
	ScrollArea::vertical().show(ui, |ui| {
		let mut panels = app.data.panels.write().expect("Panels RwLock poisoned"); // TODO only lock as write when editing
		let panels_count = panels.len();
		let metrics = app.data.metrics.read().expect("Metrics RwLock poisoned"); // TODO only lock as write when editing
		for (index, panel) in panels.iter_mut().enumerate() {
			if index > 0 {
				ui.separator();
			}
			CollapsingState::load_with_default_open(
				ctx,
				ui.make_persistent_id(format!("panel-{}-compressable", panel.id)),
				true,
			)
			.show_header(ui, |ui| {
				if app.edit {
					if ui.small_button(" + ").clicked() {
						if index > 0 {
							to_swap = Some(index); // TODO kinda jank but is there a better way?
						}
					}
					if ui.small_button(" − ").clicked() {
						if index < panels_count - 1 {
							to_swap = Some(index + 1); // TODO kinda jank but is there a better way?
						}
					}
					if ui.small_button(" × ").clicked() {
						to_delete = Some(index); // TODO kinda jank but is there a better way?
					}
					ui.separator();
				}
				panel_title_ui(ui, panel, app.edit);
			})
			.body(|ui| panel_body_ui(ui, panel, &metrics));
		}
	});
	if let Some(i) = to_delete {
		// TODO can this be done in background? idk
		let mut panels = app.data.panels.write().expect("Panels RwLock poisoned");
		if let Err(e) = app
			.data
			.storage
			.lock()
			.expect("Storage Mutex poisoned")
			.delete_panel(panels[i].id)
		{
			error!(target: "ui", "Could not delete panel : {:?}", e);
		} else {
			for metric in app
				.data
				.metrics
				.write()
				.expect("Sources RwLock poisoned")
				.iter_mut()
			{
				if metric.panel_id == panels[i].id {
					metric.panel_id = -1;
				}
			}
			panels.remove(i);
		}
	} else if let Some(i) = to_swap {
		// TODO can this be done in background? idk
		let mut panels = app.data.panels.write().expect("Panels RwLock poisoned");
		panels.swap(i - 1, i);
	}
}

pub fn panel_edit_inline_ui(ui: &mut Ui, panel: &mut Panel) {
	TextEdit::singleline(&mut panel.name)
		.hint_text("name")
		.desired_width(100.0)
		.show(ui);
}

pub fn panel_title_ui(ui: &mut Ui, panel: &mut Panel, edit: bool) { // TODO make edit UI in separate func
	ui.horizontal(|ui| {
		if edit {
			TextEdit::singleline(&mut panel.name)
				.hint_text("name")
				.desired_width(150.0)
				.show(ui);
			ui.separator();
			ui.add(Slider::new(&mut panel.height, 0..=500).text("height"));
			ui.separator();
			ui.checkbox(&mut panel.timeserie, "timeserie");
		} else {
			ui.heading(panel.name.as_str());
		}
		ui.with_layout(Layout::right_to_left(), |ui| {
			ui.horizontal(|ui| {
				ui.toggle_value(&mut panel.view_scroll, "🔒");
				ui.separator();
				if panel.limit {
					ui.add(
						DragValue::new(&mut panel.view_size)
							.speed(10)
							.suffix(" min")
							.clamp_range(0..=2147483647i32),
					);
				}
				ui.toggle_value(&mut panel.limit, "limit");
				ui.separator();
				if panel.shift {
					ui.add(
						DragValue::new(&mut panel.view_offset)
							.speed(10)
							.suffix(" min")
							.clamp_range(0..=2147483647i32),
					);
				}
				ui.toggle_value(&mut panel.shift, "offset");
				ui.separator();
				if panel.reduce {
					ui.add(
						DragValue::new(&mut panel.view_chunks)
							.speed(1)
							.prefix("x")
							.clamp_range(1..=1000), // TODO allow to average larger spans maybe?
					);
					ui.toggle_value(&mut panel.average, "avg");
				}
				ui.toggle_value(&mut panel.reduce, "reduce");
			});
		});
	});
}

pub fn panel_body_ui(ui: &mut Ui, panel: &mut Panel, metrics: &Vec<Metric>) {
	let mut p = Plot::new(format!("plot-{}", panel.name))
		.height(panel.height as f32)
		.allow_scroll(false)
		.legend(Legend::default().position(Corner::LeftTop));

	if panel.limit {
		p = p.set_margin_fraction(Vec2 { x: 0.0, y: 0.1 });
	}


	if panel.timeserie {
		if panel.view_scroll {
			let _now = (Utc::now().timestamp() as f64) - (60.0 * panel.view_offset as f64);
			p = p.include_x(_now);
			if panel.limit {
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
	let _now = Utc::now().timestamp() as f64;
	let _off = (panel.view_offset as f64) * 60.0; // TODO multiplying x60 makes sense only for timeseries
	let _size = (panel.view_size as f64) * 60.0; // TODO multiplying x60 makes sense only for timeseries
	let min_x = if panel.limit { Some(_now - _size - _off) } else { None };
	let max_x = if panel.shift { Some(_now - _off) } else { None };
	let chunk_size = if panel.reduce { Some(panel.view_chunks) } else { None };
	for metric in metrics {
		if metric.panel_id == panel.id {
			let values = metric.values(min_x, max_x, chunk_size, panel.average); 
			// if !panel.timeserie && panel.view_scroll && values.len() > 0 {
			// 	let l = values.len() - 1;
			// 	p = p.include_x(values[0].x)
			// 		.include_x(values[l].x)
			// 		.include_y(values[0].y)
			// 		.include_y(values[l].y);
			// }
			lines.push(
				Line::new(Values::from_values(values))
					.name(metric.name.as_str())
					.color(metric.color)
			);
		}
	}

	p.show(ui, |plot_ui| {
		for line in lines {
			plot_ui.line(line);
		}
	});
}
