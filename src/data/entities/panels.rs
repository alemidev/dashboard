//! SeaORM Entity. Generated by sea-orm-codegen 0.10.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "panels")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = true)]
	pub id: i64,
	pub name: String,
	pub view_scroll: bool,
	pub view_size: i32,
	pub timeserie: bool,
	pub height: i32,
	pub position: i32,
	pub reduce_view: bool,
	pub view_chunks: i32,
	pub view_offset: i32,
	pub average_view: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl Related<super::metrics::Entity> for Entity {
	fn to() -> RelationDef {
		super::panel_metric::Relation::Metric.def()
	}

	fn via() -> Option<RelationDef> {
		Some(super::panel_metric::Relation::Panel.def().rev())
	}
}

impl ActiveModelBehavior for ActiveModel {}

impl Default for Model {
	fn default() -> Self {
		Model {
			id: 0,
			name: "".into(),
			view_scroll: true,
			view_size: 1000,
			timeserie: true,
			height: 100,
			position: 0,
			reduce_view: false,
			view_chunks: 10,
			view_offset: 0,
			average_view: true,
		}
	}

}
