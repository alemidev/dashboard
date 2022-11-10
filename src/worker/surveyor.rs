use std::sync::Arc;

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
	index: usize,
) {
	let mut last_activation = Utc::now().timestamp();
	let mut last_fetch = 0;
	let mut sources = vec![];
	let mut metrics = Arc::new(vec![]);

	while *run.borrow() {
		// sleep until next activation
		let delta_time = (interval as i64) - (Utc::now().timestamp() - last_activation);
		if delta_time > 0 {
			tokio::time::sleep(std::time::Duration::from_secs(delta_time as u64)).await;
		}
		last_activation = Utc::now().timestamp();

		if Utc::now().timestamp() - last_fetch > cache_time {
			// TODO do both concurrently
			match entities::sources::Entity::find().all(&db).await {
				Ok(srcs) => sources = srcs,
				Err(e) => {
					error!(target: "surveyor", "[{}] Could not fetch sources: {:?}", index, e);
					continue;
				}
			}
			match entities::metrics::Entity::find().all(&db).await {
				Ok(mtrcs) => metrics = Arc::new(mtrcs),
				Err(e) => {
					error!(target: "surveyor", "[{}] Could not fetch metrics: {:?}", index, e);
					continue;
				}
			}
			last_fetch = Utc::now().timestamp();
		}

		for source in sources.iter_mut() {
			if !source.enabled || !source.ready() {
				continue;
			}

			let metrics_snapshot = metrics.clone();
			let db_clone = db.clone();
			let source_clone = source.clone();
			let now = Utc::now().timestamp();
			source.last_update = now; // TODO kinda meh
			// we set this before knowing about fetch result, to avoid re-running a fetch
			// next time this loop runs. But the task only sets last_update on db if fetch succeeds,
			// so if an error happens the client and server last_update fields will differ until fetched
			// again. This could be avoided by keeping track of which threads are trying which sources,
			// but also only trying to fetch at certain intervals to stay aligned might be desirable.
			tokio::spawn(async move {
				match fetch(&source_clone.url).await {
					Ok(res) => {
						if let Err(e) = entities::sources::Entity::update(
							entities::sources::ActiveModel{id: Set(source_clone.id), last_update: Set(now), ..Default::default()}
						).exec(&db_clone).await {
							error!(target: "surveyor", "[{}] Failed setting last_update ({:?}) for source {:?} but successfully fetched '{}', aborting", index, e, source_clone, res);
							return;
						}
						let now = Utc::now().timestamp() as f64;
						for metric in metrics_snapshot.iter().filter(|x| source_clone.id == x.source_id) {
							match metric.extract(&res) {
								// note that Err and None mean different things: Err for broken queries, None for
								// missing values. Only first one is reported
								Ok(value) => {
									if let Some(v) = value {
										if let Err(e) = entities::points::Entity::insert(
											entities::points::ActiveModel {
												id: NotSet, metric_id: Set(metric.id), x: Set(now), y: Set(v),
										}).exec(&db_clone).await {
											error!(target: "surveyor", "[{}] Could not insert record ({},{}) : {:?}", index, now, v, e);
										}
									}
								},
								Err(e) => error!(target: "surveyor", "[{}] Failed extracting '{}' from {}: {:?}", index, metric.name, source_clone.name, e),
							}
						}
					},
					Err(e) => error!(target: "surveyor", "[{}] Failed fetching {}: {:?}", index, source_clone.name, e),
				}
			});
		}
	}
}
