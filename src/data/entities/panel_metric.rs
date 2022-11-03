use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "panel_metric")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = true)]
	pub id: i64,
	pub panel_id: i64,
	pub metric_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::panels::Entity",
		from = "Column::PanelId",
		to = "super::panels::Column::Id"
	)]
	Panel,
	
	#[sea_orm(
		belongs_to = "super::metrics::Entity",
		from = "Column::MetricId",
		to = "super::metrics::Column::Id"
	)]
	Metric,
}

impl ActiveModelBehavior for ActiveModel {}
