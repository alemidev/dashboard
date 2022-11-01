use chrono::Utc;
use sea_orm::{DatabaseConnection, ActiveValue::NotSet, Set, EntityTrait};
use tokio::sync::watch;
use tracing::error;

use crate::data::{entities, FetchError};

async fn fetch(url: &str) -> Result<serde_json::Value, FetchError> {
	Ok(reqwest::get(url).await?.json().await?)
}

pub async fn surveyor_loop(
	db: DatabaseConnection,
	interval:i64,
	cache_time:i64,
	run: watch::Receiver<bool>,
) {
	let mut last_check = Utc::now().timestamp();
	let mut last_fetch = 0;
	let mut sources = vec![];
	let mut metrics = vec![];
	while *run.borrow() {
		// sleep until next activation
		let delta_time = (interval as i64) - (Utc::now().timestamp() - last_check);
		if delta_time > 0 {
			tokio::time::sleep(std::time::Duration::from_secs(delta_time as u64)).await;
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
