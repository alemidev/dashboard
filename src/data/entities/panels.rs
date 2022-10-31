//! SeaORM Entity. Generated by sea-orm-codegen 0.10.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "panels")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = true)]
	pub id: i32,
	pub name: String,
	pub view_scroll: bool,
	pub view_size: i32,
	pub timeserie: bool,
	pub height: i32,
	pub limit_view: bool,
	pub position: i32,
	pub reduce_view: bool,
	pub view_chunks: i32,
	pub shift_view: bool,
	pub view_offset: i32,
	pub average_view: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

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
			limit_view: true,
			position: 0,
			reduce_view: false,
			view_chunks: 10,
			shift_view: false,
			view_offset: 0,
			average_view: true,
		}
	}

}