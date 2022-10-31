use chrono::Utc;
use sea_orm::{DatabaseConnection, ActiveValue::NotSet, Set, EntityTrait, Condition, ColumnTrait, QueryFilter};
use tokio::sync::watch;
use tracing::{error, info};
use std::collections::VecDeque;

use super::data::{entities, FetchError};

async fn fetch(url: &str) -> Result<serde_json::Value, FetchError> {
	Ok(reqwest::get(url).await?.json().await?)
}

pub async fn surveyor_loop(
	db: DatabaseConnection,
	interval:i64,
	cache_time:i64,
) {
	let mut last_check = Utc::now().timestamp();
	let mut last_fetch = 0;
	let mut sources = vec![];
	let mut metrics = vec![];
	loop {
		// sleep until next activation
		let delta_time = (interval as i64) - (Utc::now().timestamp() - last_check);
		if delta_time > 0 {
			std::thread::sleep(std::time::Duration::from_secs(delta_time as u64));
		}
		last_check = Utc::now().timestamp();

		if Utc::now().timestamp() - last_fetch > cache_time {
			// TODO do both concurrently
			let res = tokio::join!(
				entities::sources::Entity::find().all(&db),
				entities::metrics::Entity::find().all(&db)
			);
			sources = res.0.unwrap();
			metrics = res.1.unwrap();
			last_fetch = Utc::now().timestamp();
		}

		for source in sources.iter_mut() {
			if !source.enabled || !source.ready() {
				continue;
			}

			// source.last_fetch = Utc::now(); // TODO! do it in background threads again!
			// tokio::spawn(async move {
				match fetch(&source.url).await {
					Ok(res) => {
						let now = Utc::now().timestamp();
						entities::sources::Entity::update(
							entities::sources::ActiveModel{id: Set(source.id), last_update: Set(now), ..Default::default()}
						).exec(&db).await.unwrap();
						source.last_update = now;
						for metric in metrics.iter().filter(|x| source.id == x.source_id) {
							match metric.extract(&res) {
								Ok(v) => {
									entities::points::Entity::insert(
										entities::points::ActiveModel {
											id: NotSet, metric_id: Set(metric.id), x: Set(v.x), y: Set(v.y),
									}).exec(&db).await.unwrap();
								},
								Err(e) => error!(target: "worker", "Failed extracting '{}' from {}: {:?}", metric.name, source.name, e),
							}
						}
					},
					Err(e) => error!(target: "worker", "Failed fetching {}: {:?}", source.name, e),
				}
				// source.last_fetch = Utc::now(); // TODO!
			// });

		}

		// if let Ok(meta) = std::fs::metadata(state.file_path.clone()) {
		// 	let mut fsize = state.file_size.write().expect("File Size RwLock poisoned");
		// 	*fsize = meta.len();
		// } // ignore errors

		// ctx.request_repaint();
	}
}

pub async fn visualizer_loop(
	db: DatabaseConnection,
	interval: u64,
	cache_time: i64,
	panels_tx: watch::Sender<Vec<entities::panels::Model>>,
	sources_tx: watch::Sender<Vec<entities::sources::Model>>,
	metrics_tx: watch::Sender<Vec<entities::metrics::Model>>,
	points_tx: watch::Sender<Vec<entities::points::Model>>,
	view_rx: watch::Receiver<i64>,
) {
	let mut points : VecDeque<entities::points::Model> = VecDeque::new();
	let mut last_fetch = 0;

	let mut width = *view_rx.borrow() * 60; // TODO it's in minutes somewhere...
	let mut lower = Utc::now().timestamp() - width;

	let mut changes;

	loop {
		if Utc::now().timestamp() - last_fetch >= cache_time {
			panels_tx.send(entities::panels::Entity::find().all(&db).await.unwrap()).unwrap();
			sources_tx.send(entities::sources::Entity::find().all(&db).await.unwrap()).unwrap();
			metrics_tx.send(entities::metrics::Entity::find().all(&db).await.unwrap()).unwrap();
			last_fetch = Utc::now().timestamp();
			info!(target: "worker", "Updated panels, sources and metrics");
		}

		changes = false;
		let now = Utc::now().timestamp();
		let new_width = *view_rx.borrow() * 60; // TODO it's in minutes somewhere...

		if new_width != width {
			let mut lower_points = entities::points::Entity::find()
				.filter(
					Condition::all()
						.add(entities::points::Column::X.gte(now - new_width))
						.add(entities::points::Column::X.lte(now - width))
				)
				.all(&db)
				.await.unwrap();
			lower_points.reverse(); // TODO wasteful!
			for p in lower_points {
				points.push_front(p);
				changes = true;
			}
		}

		width = new_width;

		let new_points = entities::points::Entity::find()
			.filter(
				Condition::all()
					.add(entities::points::Column::X.gte(lower as f64))
			)
			.all(&db)
			.await.unwrap();

		lower = Utc::now().timestamp();
		while let Some(p) = points.get(0) {
			if (p.x as i64) >= lower - (*view_rx.borrow() * 60) {
				break;
			}
			points.pop_front();
			changes = true;
		}
		for p in new_points {
			points.push_back(p);
			changes = true;
		}

		if changes {
			points_tx.send(points.clone().into()).unwrap();
		}

		tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
	}
}
