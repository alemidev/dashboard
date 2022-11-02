//! SeaORM Entity. Generated by sea-orm-codegen 0.10.1

use chrono::Utc;
use eframe::egui::plot::PlotPoint;
use sea_orm::entity::prelude::*;

use crate::data::FetchError;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "metrics")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
	pub id: i64,
	pub name: String,
	pub source_id: i64,
	pub query_x: String,
	pub query_y: String,
	pub panel_id: i64,
	pub color: i32,
	pub position: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}


impl Model {
	pub fn extract(&self, value: &serde_json::Value) -> Result<PlotPoint, FetchError> {
		let x: f64;
		if self.query_x.len() > 0 {
			x = jql::walker(value, self.query_x.as_str())?
				.as_f64()
				.ok_or(FetchError::JQLError("X query is null".to_string()))?; // TODO what if it's given to us as a string?
		} else {
			x = Utc::now().timestamp() as f64;
		}
		let y = jql::walker(value, self.query_y.as_str())?
			.as_f64()
			.ok_or(FetchError::JQLError("Y query is null".to_string()))?;
		Ok(PlotPoint { x, y })
	}
}

impl Default for Model {
	fn default() -> Self {
		Model {
			id: 0,
			name: "".into(),
			source_id: 0,
			query_x: "".into(),
			query_y: "".into(),
			panel_id: 0,
			color: 0,
			position: 0,
		}
	}

}
