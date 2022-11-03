use sea_orm::entity::prelude::*;
use chrono::Utc;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "sources")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
	pub id: i64,
	pub name: String,
	pub enabled: bool,
	pub url: String,
	pub interval: i32,
	pub last_update: i64,
	pub position: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(has_many = "super::metrics::Entity")]
	Metric,
}

impl Related<super::metrics::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Metric.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
	pub fn cooldown(&self) -> i64 {
		let elapsed = Utc::now().timestamp() - self.last_update;
		(self.interval as i64) - elapsed
	}

	pub fn ready(&self) -> bool {
		self.cooldown() <= 0

	}
}

impl Default for Model {
	fn default() -> Self {
		Model {
			id: 0,
			name: "".into(),
			enabled: false,
			url: "".into(),
			interval: 60,
			last_update: 0,
			position: 0,
		}
	}
}
