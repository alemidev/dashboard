use sea_orm::entity::prelude::*;

use eframe::egui::plot::PlotPoint;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "points")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
	pub id: i64,
	pub metric_id: i64,
	pub x: f64,
	pub y: f64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::metrics::Entity",
		from = "Column::MetricId",
		to = "super::metrics::Column::Id"
	)]
	Metric,
}

impl Related<super::metrics::Entity> for Entity {
	fn to() -> RelationDef { Relation::Metric.def() }
}

impl ActiveModelBehavior for ActiveModel {}

impl Into<PlotPoint> for Model {
	fn into(self) -> PlotPoint {
		PlotPoint { x: self.x, y: self.y }
	}
}
