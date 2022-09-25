use axum::{
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde_json::json;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
	#[error("database error: {0}")]
	Database(#[from] sea_orm::error::DbErr),
	#[error("io error: {0}")]
	Io(#[from] std::io::Error),
}

impl IntoResponse for Error {
	fn into_response(self) -> Response {
		let (status, error_msg) = match self {
			Self::Database(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
			Self::Io(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
		};
		let body = Json(json!({
			"status": status.as_u16(),
			"error": error_msg,
		}));

		(status, body).into_response()
	}
}

pub type Result<T> = std::result::Result<T, self::Error>;
