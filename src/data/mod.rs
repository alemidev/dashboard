pub mod entities;

use std::num::ParseFloatError;

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
