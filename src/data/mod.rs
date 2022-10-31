pub mod entities;

use std::num::ParseFloatError;

use sea_orm::{DatabaseConnection, EntityTrait};

#[derive(Debug)]
pub enum FetchError {
	ReqwestError(reqwest::Error),
	IoError(std::io::Error),
	JQLError(String),
	ParseFloatError(ParseFloatError),
	DbError(sea_orm::DbErr),
}

impl From<reqwest::Error> for FetchError {
	fn from(e: reqwest::Error) -> Self {
		FetchError::ReqwestError(e)
	}
}
impl From<std::io::Error> for FetchError {
	fn from(e: std::io::Error) -> Self {
		FetchError::IoError(e)
	}
}
impl From<String> for FetchError {
	// TODO wtf? why does JQL error as a String?
	fn from(e: String) -> Self {
		FetchError::JQLError(e)
	}
}
impl From<ParseFloatError> for FetchError {
	fn from(e: ParseFloatError) -> Self {
		FetchError::ParseFloatError(e)
	}
}
impl From<sea_orm::DbErr> for FetchError {
	fn from(e: sea_orm::DbErr) -> Self {
		FetchError::DbError(e)
	}
}

#[allow(dead_code)]
pub struct ApplicationState {
	// pub run: bool,
	db: DatabaseConnection,
	pub panels: Vec<entities::panels::Model>,
	pub sources: Vec<entities::sources::Model>,
	pub metrics: Vec<entities::metrics::Model>,
	last_fetch: i64,
	// pub diagnostics: RwLock<Vec<String>>,
}

#[allow(dead_code)]
impl ApplicationState {
	pub fn new(db: DatabaseConnection) -> Result<ApplicationState, FetchError> {
		return Ok(ApplicationState {
			db,
			panels: vec![],
			sources: vec![],
			metrics: vec![],
			last_fetch: 0,
		});
	}

	pub async fn fetch(&mut self) -> Result<(), sea_orm::DbErr> {
		self.panels = entities::panels::Entity::find().all(&self.db).await?;
		self.sources = entities::sources::Entity::find().all(&self.db).await?;
		self.metrics = entities::metrics::Entity::find().all(&self.db).await?;
		self.last_fetch = chrono::Utc::now().timestamp();
		Ok(())
	}

	pub fn age(&self) -> i64 {
		chrono::Utc::now().timestamp() - self.last_fetch
	}
}
