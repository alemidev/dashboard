use chrono::Utc;
use sea_orm::{TransactionTrait, DatabaseConnection, EntityTrait, Condition, ColumnTrait, QueryFilter, Set, QueryOrder, Order, ActiveModelTrait, ActiveValue::{NotSet, self}, Database, DbErr};
use tokio::sync::{watch, mpsc};
use tracing::{info, error, warn};
use std::collections::VecDeque;

use crate::data::{entities, FetchError};

#[derive(Clone)]
pub struct AppStateView {
	pub panels:       watch::Receiver<Vec<entities::panels::Model>>,
	pub sources:      watch::Receiver<Vec<entities::sources::Model>>,
	pub metrics:      watch::Receiver<Vec<entities::metrics::Model>>,
	pub panel_metric: watch::Receiver<Vec<entities::panel_metric::Model>>,
	pub points:       watch::Receiver<Vec<entities::points::Model>>,
	pub flush:        mpsc::Sender<()>,
	pub op:           mpsc::Sender<BackgroundAction>,
}

impl AppStateView {
	pub async fn request_flush(&self) -> bool {
		match self.flush.send(()).await {
			Ok(_) => true,
			Err(_) => false,
		}
	}
}

struct AppStateTransmitters {
	panels:       watch::Sender<Vec<entities::panels::Model>>,
	sources:      watch::Sender<Vec<entities::sources::Model>>,
	metrics:      watch::Sender<Vec<entities::metrics::Model>>,
	points:       watch::Sender<Vec<entities::points::Model>>,
	panel_metric: watch::Sender<Vec<entities::panel_metric::Model>>,
}

pub struct AppState {
	tx: AppStateTransmitters,

	db_uri:       mpsc::Receiver<String>,

	panels:       Vec<entities::panels::Model>,
	sources:      Vec<entities::sources::Model>,
	metrics:      Vec<entities::metrics::Model>,
	panel_metric: Vec<entities::panel_metric::Model>,
	last_refresh: i64,

	points:  VecDeque<entities::points::Model>,
	last_check: i64,

	flush: mpsc::Receiver<()>,
	op: mpsc::Receiver<BackgroundAction>,

	interval: i64,
	cache_age: i64,

	width: watch::Receiver<i64>,
	last_width: i64,

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
		db_uri: mpsc::Receiver<String>,
		interval: i64,
		cache_age: i64,
	) -> Result<AppState, FetchError> {
		let (panel_tx, panel_rx) = watch::channel(vec![]);
		let (source_tx, source_rx) = watch::channel(vec![]);
		let (metric_tx, metric_rx) = watch::channel(vec![]);
		let (point_tx, point_rx) = watch::channel(vec![]);
		let (panel_metric_tx, panel_metric_rx) = watch::channel(vec![]);
		// let (view_tx, view_rx) = watch::channel(0);
		let (flush_tx, flush_rx) = mpsc::channel(10);
		let (op_tx, op_rx) = mpsc::channel(100);

		Ok(AppState {
			panels: vec![],
			sources: vec![],
			metrics: vec![],
			panel_metric: vec![],
			last_refresh: 0,
			points: VecDeque::new(),
			last_check: 0,
			last_width: 0,
			flush: flush_rx,
			op: op_rx,
			view: AppStateView {
				panels: panel_rx,
				sources: source_rx,
				metrics: metric_rx,
				points: point_rx,
				panel_metric: panel_metric_rx,
				flush: flush_tx,
				op: op_tx,
			},
			tx: AppStateTransmitters {
				panels: panel_tx,
				sources: source_tx,
				metrics: metric_tx,
				points: point_tx,
				panel_metric: panel_metric_tx,
			},
			width,
			db_uri,
			interval,
			cache_age,
		})
	}

	pub fn view(&self) -> AppStateView {
		self.view.clone()
	}

	pub async fn fetch(&mut self, db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
		// TODO parallelize all this stuff
		self.panels = entities::panels::Entity::find()
			.order_by(entities::panels::Column::Position, Order::Asc)
			.order_by(entities::panels::Column::Id, Order::Asc)
			.all(db).await?;
		if let Err(e) = self.tx.panels.send(self.panels.clone()) { 
			error!(target: "state-manager", "Could not send panels update: {:?}", e);
		}

		self.sources = entities::sources::Entity::find()
			.order_by(entities::sources::Column::Position, Order::Asc)
			.order_by(entities::sources::Column::Id, Order::Asc)
			.all(db).await?;
		if let Err(e) = self.tx.sources.send(self.sources.clone()) {
			error!(target: "state-manager", "Could not send sources update: {:?}", e);
		}

		self.metrics = entities::metrics::Entity::find()
			.order_by(entities::metrics::Column::Position, Order::Asc)
			.order_by(entities::metrics::Column::SourceId, Order::Asc)
			.order_by(entities::metrics::Column::Id, Order::Asc)
			.all(db).await?;
		if let Err(e) = self.tx.metrics.send(self.metrics.clone()) {
			error!(target: "state-manager", "Could not send metrics update: {:?}", e);
		}

		self.panel_metric = entities::panel_metric::Entity::find()
			.all(db).await?;
		if let Err(e) = self.tx.panel_metric.send(self.panel_metric.clone()) {
			error!(target: "state-manager", "Could not send panel-metric update: {:?}", e);
		}

		self.last_refresh = chrono::Utc::now().timestamp();
		Ok(())
	}

	pub fn _cache_age(&self) -> i64 {
		chrono::Utc::now().timestamp() - self.last_refresh
	}

	pub async fn parse_op(&mut self, op:BackgroundAction, db: &DatabaseConnection) -> Result<(), DbErr> {
		match op {
			BackgroundAction::UpdateAllPanels { panels } => {
				// TODO this is kinda rough, can it be done better?
				let pnls = panels.clone();
				if let Err(e) = db.transaction::<_, (), sea_orm::DbErr>(|txn| {
					Box::pin(async move {
						entities::panels::Entity::delete_many().exec(txn).await?;
						entities::panels::Entity::insert_many(
							pnls.iter().map(|v| entities::panels::ActiveModel{
								id: Set(v.id),
								name: Set(v.name.clone()),
								view_scroll: Set(v.view_scroll),
								view_size: Set(v.view_size),
								height: Set(v.height),
								position: Set(v.position),
								reduce_view: Set(v.reduce_view),
								view_chunks: Set(v.view_chunks),
								view_offset: Set(v.view_offset),
								average_view: Set(v.average_view),
							}).collect::<Vec<entities::panels::ActiveModel>>()
						).exec(txn).await?;
						Ok(())
					})
				}).await {
					error!(target: "state-manager", "Could not update panels on database: {:?}", e);
				} else {
					if let Err(e) = self.tx.panels.send(panels.clone()) {
						error!(target: "state-manager", "Could not send panels update: {:?}", e);
					}
					self.panels = panels;
				}
			},
			BackgroundAction::UpdatePanel { panel, metrics } => {
				let panel_id = match panel.id {
					ActiveValue::Unchanged(pid) => Some(pid),
					_ => None,
				};
				let op = if panel.id == NotSet { panel.insert(db) } else { panel.update(db) };
				op.await?;
				// TODO chained if is trashy
				if let Some(panel_id) = panel_id {
					if let Err(e) = db.transaction::<_, (), sea_orm::DbErr>(|txn| {
						Box::pin(async move {
							entities::panel_metric::Entity::delete_many()
								.filter(
									Condition::all()
										.add(entities::panel_metric::Column::PanelId.eq(panel_id))
								)
								.exec(txn).await?;
							entities::panel_metric::Entity::insert_many(metrics).exec(txn).await?;
							Ok(())
						})
					}).await {
						error!(target: "state-manager", "Could not update panels on database: {:?}", e);
					}
				} else {
					self.view.request_flush().await;
				}
			},
			BackgroundAction::UpdateSource { source } => {
				let op = if source.id == NotSet { source.insert(db) } else { source.update(db) };
				op.await?;
				self.view.request_flush().await;
			},
			BackgroundAction::UpdateMetric { metric } => {
				let op = if metric.id == NotSet { metric.insert(db) } else { metric.update(db) };
				if let Err(e) = op.await {
					error!(target: "state-manager", "Could not update metric: {:?}", e);
				} else {
					self.view.request_flush().await;
				}
			},
			// _ => todo!(),
		}
		Ok(())
	}

	pub async fn flush_data(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
		let now = Utc::now().timestamp();
		self.fetch(db).await?;
		self.last_width = *self.width.borrow() * 60; // TODO it's in minutes somewhere...
		self.points = entities::points::Entity::find()
			.filter(
				Condition::all()
					.add(entities::points::Column::X.gte((now - self.last_width) as f64))
					.add(entities::points::Column::X.lte(now as f64))
			)
			.order_by(entities::points::Column::X, Order::Asc)
			.all(db)
			.await?.into();
		if let Err(e) = self.tx.points.send(self.points.clone().into()) {
			warn!(target: "state-manager", "Could not send new points: {:?}", e); // TODO should be an err?
		}
		self.last_check = now;
		Ok(())
	}

	pub async fn update_points(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
		let mut changes = false;
		let now = Utc::now().timestamp();
		let new_width = *self.width.borrow() * 60; // TODO it's in minutes somewhere...
	
		// fetch previous points
		if new_width != self.last_width {
			let previous_points = entities::points::Entity::find()
				.filter(
					Condition::all()
						.add(entities::points::Column::X.gte(now - new_width))
						.add(entities::points::Column::X.lte(now - self.last_width))
				)
				.order_by(entities::points::Column::X, Order::Desc)
				.all(db)
				.await?;
			for p in previous_points {
				self.points.push_front(p);
				changes = true;
			}
		}
	
		// fetch new points, use last_width otherwise it fetches same points as above
		let lower_bound = std::cmp::max(self.last_check, now - self.last_width);
		let new_points = entities::points::Entity::find()
			.filter(
				Condition::all()
					.add(entities::points::Column::X.gte(lower_bound as f64))
					.add(entities::points::Column::X.lte(now as f64))
			)
			.order_by(entities::points::Column::X, Order::Asc)
			.all(db)
			.await?;
	
		for p in new_points {
			self.points.push_back(p);
			changes = true;
		}
	
		// remove old points
		while let Some(p) = self.points.get(0) {
			if (p.x as i64) >= now - new_width {
				break;
			}
			self.points.pop_front();
			changes = true;
		}
	
		// update
		self.last_width = new_width;
		self.last_check = now;
		if changes {
			if let Err(e) = self.tx.points.send(self.points.clone().into()) {
				warn!(target: "state-manager", "Could not send changes to main thread: {:?}", e);
			}
		}
		Ok(())
	}

	pub async fn worker(mut self, run:watch::Receiver<bool>) {
		let mut now;
		let Some(first_db_uri) = self.db_uri.recv().await else {
			warn!(target: "state-manager", "No initial database URI, skipping first connection");
			return;
		};

		let mut db = Database::connect(first_db_uri.clone()).await.unwrap();

		info!(target: "state-manager", "Connected to '{}'", first_db_uri);

		while *run.borrow() {
			now = Utc::now().timestamp();
			tokio::select!{
				res = self.db_uri.recv() => {
					match res {
						Some(uri) => {
							match Database::connect(uri.clone()).await {
								Ok(new_db) => {
									info!("Connected to '{}'", uri);
									db = new_db;
									self.last_check = 0;
									self.last_refresh = 0;
								},
								Err(e) => error!(target: "state-manager", "Could not connect to db: {:?}", e),
							};
						},
						None => { error!(target: "state-manager", "URI channel closed"); break; },
					}
				},
				res = self.op.recv() => {
					match res {
						Some(op) => match self.parse_op(op, &db).await {
							Ok(()) => { },
							Err(e) => error!(target: "state-manager", "Failed executing operation: {:?}", e),
						},
						None => { error!(target: "state-manager", "Operations channel closed"); break; },
					}
				}
				res = self.flush.recv() => {
					match res {
						Some(()) => match self.flush_data(&db).await {
							Ok(()) => { },
							Err(e) => error!(target: "state-manager", "Could not flush away current data: {:?}", e),
						},
						None => { error!(target: "state-manager", "Flush channel closed"); break; },
					}
				},
				_ = sleep(self.cache_age - (now - self.last_refresh)) => {
					if let Err(e) = self.fetch(&db).await {
						error!(target: "state-manager", "Could not fetch from db: {:?}", e);
					}
				},
				_ = sleep(self.interval - (now - self.last_check)) => {
					if let Err(e) = self.update_points(&db).await {
						error!(target: "state-manager", "Could not update points: {:?}", e);
					}
				}
			}
		}
	}
}

#[derive(Debug)]
pub enum BackgroundAction {
	UpdateAllPanels { panels: Vec<entities::panels::Model> },
	UpdatePanel     { panel : entities::panels::ActiveModel, metrics: Vec<entities::panel_metric::ActiveModel> },
	UpdateSource    { source: entities::sources::ActiveModel },
	UpdateMetric    { metric: entities::metrics::ActiveModel },
	// InsertPanel     { panel : entities::panels::ActiveModel },
	// InsertSource    { source: entities::sources::ActiveModel },
	// InsertMetric    { metric: entities::metrics::ActiveModel },
}
