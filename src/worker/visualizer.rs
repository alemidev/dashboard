use chrono::Utc;
use sea_orm::{DatabaseConnection, EntityTrait, Condition, ColumnTrait, QueryFilter, Set};
use tokio::sync::{watch, mpsc};
use tracing::info;
use std::collections::VecDeque;

use crate::data::{entities, FetchError};

#[derive(Clone)]
pub struct AppStateView {
	pub panels:  watch::Receiver<Vec<entities::panels::Model>>,
	pub sources: watch::Receiver<Vec<entities::sources::Model>>,
	pub metrics: watch::Receiver<Vec<entities::metrics::Model>>,
	pub points:  watch::Receiver<Vec<entities::points::Model>>,
	pub flush:   mpsc::Sender<()>,
	pub op:      mpsc::Sender<BackgroundAction>,
}

impl AppStateView {
	pub async fn _request_flush(&self) -> bool {
		match self.flush.send(()).await {
			Ok(_) => true,
			Err(_) => false,
		}
	}
}

struct AppStateTransmitters {
	panels:  watch::Sender<Vec<entities::panels::Model>>,
	sources: watch::Sender<Vec<entities::sources::Model>>,
	metrics: watch::Sender<Vec<entities::metrics::Model>>,
	points:  watch::Sender<Vec<entities::points::Model>>,
}

pub struct AppState {
	tx: AppStateTransmitters,

	panels:  Vec<entities::panels::Model>,
	sources: Vec<entities::sources::Model>,
	metrics: Vec<entities::metrics::Model>,
	last_refresh: i64,

	points:  VecDeque<entities::points::Model>,
	last_check: i64,

	flush: mpsc::Receiver<()>,
	op: mpsc::Receiver<BackgroundAction>,

	interval: i64,
	cache_age: i64,

	width: watch::Receiver<i64>,

	view: AppStateView,
}

async fn sleep(t:i64) {
	if t > 0 {
		tokio::time::sleep(std::time::Duration::from_secs(t as u64)).await
	}
}

impl AppState {
	pub fn new(
		width: watch::Receiver<i64>,
		interval: i64,
		cache_age: i64,
	) -> Result<AppState, FetchError> {
		let (panel_tx, panel_rx) = watch::channel(vec![]);
		let (source_tx, source_rx) = watch::channel(vec![]);
		let (metric_tx, metric_rx) = watch::channel(vec![]);
		let (point_tx, point_rx) = watch::channel(vec![]);
		// let (view_tx, view_rx) = watch::channel(0);
		let (flush_tx, flush_rx) = mpsc::channel(10);
		let (op_tx, op_rx) = mpsc::channel(100);

		Ok(AppState {
			panels: vec![],
			sources: vec![],
			metrics: vec![],
			last_refresh: 0,
			points: VecDeque::new(),
			last_check: 0,
			flush: flush_rx,
			op: op_rx,
			view: AppStateView {
				panels: panel_rx,
				sources: source_rx,
				metrics: metric_rx,
				points: point_rx,
				flush: flush_tx,
				op: op_tx,
			},
			tx: AppStateTransmitters {
				panels: panel_tx,
				sources: source_tx,
				metrics: metric_tx,
				points: point_tx,
			},
			width,
			interval,
			cache_age,
		})
	}

	pub fn view(&self) -> AppStateView {
		self.view.clone()
	}

	pub async fn fetch(&mut self, db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
		// TODO parallelize all this stuff
		self.panels = entities::panels::Entity::find().all(db).await?;
		self.tx.panels.send(self.panels.clone()).unwrap();
		self.sources = entities::sources::Entity::find().all(db).await?;
		self.tx.sources.send(self.sources.clone()).unwrap();
		self.metrics = entities::metrics::Entity::find().all(db).await?;
		self.tx.metrics.send(self.metrics.clone()).unwrap();
		info!(target: "worker", "Updated panels, sources and metrics");
		self.last_refresh = chrono::Utc::now().timestamp();
		Ok(())
	}

	pub fn _cache_age(&self) -> i64 {
		chrono::Utc::now().timestamp() - self.last_refresh
	}

	pub async fn worker(mut self, db: DatabaseConnection, run:watch::Receiver<bool>) {
		let mut width = *self.width.borrow() * 60; // TODO it's in minutes somewhere...
		let mut last = Utc::now().timestamp() - width;
	
		while *run.borrow() {
			let now = Utc::now().timestamp();
			tokio::select!{
				op = self.op.recv() => {
					match op {
						Some(op) => {
							match op {
								BackgroundAction::UpdateAllPanels { panels } => {
									// TODO this is kinda rough, can it be done better?
									entities::panels::Entity::delete_many().exec(&db).await.unwrap();
									entities::panels::Entity::insert_many(
										panels.iter().map(|v| entities::panels::ActiveModel{
											id: Set(v.id),
											name: Set(v.name.clone()),
											view_scroll: Set(v.view_scroll),
											view_size: Set(v.view_size),
											timeserie: Set(v.timeserie),
											height: Set(v.height),
											limit_view: Set(v.limit_view),
											position: Set(v.position),
											reduce_view: Set(v.reduce_view),
											view_chunks: Set(v.view_chunks),
											shift_view: Set(v.shift_view),
											view_offset: Set(v.view_offset),
											average_view: Set(v.average_view),
										}).collect::<Vec<entities::panels::ActiveModel>>()
									).exec(&db).await.unwrap();
									self.tx.panels.send(panels.clone()).unwrap();
									self.panels = panels;
								},
								// _ => todo!(),
							}
						},
						None => {},
					}
				},
				_ = self.flush.recv() => {
					let now = Utc::now().timestamp();
					self.fetch(&db).await.unwrap();
					let new_width = *self.width.borrow() * 60; // TODO it's in minutes somewhere...
					self.points = entities::points::Entity::find()
						.filter(
							Condition::all()
								.add(entities::points::Column::X.gte((now - new_width) as f64))
								.add(entities::points::Column::X.lte(now as f64))
						)
						.all(&db)
						.await.unwrap().into();
					self.tx.points.send(self.points.clone().into()).unwrap();
					last = Utc::now().timestamp();
					info!(target: "worker", "Reloaded points");
				},
				_ = sleep(self.cache_age - (now - self.last_refresh)) => self.fetch(&db).await.unwrap(),
				_ = sleep(self.interval - (now - self.last_check)) => {
					let mut changes = false;
					let now = Utc::now().timestamp();
					let new_width = *self.width.borrow() * 60; // TODO it's in minutes somewhere...
	
					// fetch previous points
					if new_width != width {
						let mut previous_points = entities::points::Entity::find()
							.filter(
								Condition::all()
									.add(entities::points::Column::X.gte(now - new_width))
									.add(entities::points::Column::X.lte(now - width))
							)
							.all(&db)
							.await.unwrap();
						info!(target: "worker", "Fetched {} previous points", previous_points.len());
						previous_points.reverse(); // TODO wasteful!
						for p in previous_points {
							self.points.push_front(p);
							changes = true;
						}
					}
	
					// fetch new points
					let new_points = entities::points::Entity::find()
						.filter(
							Condition::all()
								.add(entities::points::Column::X.gte(last as f64))
								.add(entities::points::Column::X.lte(now as f64))
						)
						.all(&db)
						.await.unwrap();
					info!(target: "worker", "Fetched {} new points", new_points.len());
	
					for p in new_points {
						self.points.push_back(p);
						changes = true;
					}
	
					// remove old points
					while let Some(p) = self.points.get(0) {
						if (p.x as i64) >= now - (*self.width.borrow() * 60) {
							break;
						}
						self.points.pop_front();
						changes = true;
					}
	
					// update
					last = now;
					width = new_width;
					self.last_check = now;
					if changes {
						self.tx.points.send(self.points.clone().into()).unwrap();
					}
				},
			};
		}
	}
}

#[derive(Debug)]
pub enum BackgroundAction {
	UpdateAllPanels { panels: Vec<entities::panels::Model> },
	// UpdatePanel     { panel : entities::panels::ActiveModel },
	// UpdateSource    { source: entities::sources::ActiveModel },
	// UpdateMetric    { metric: entities::metrics::ActiveModel },
}
