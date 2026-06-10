use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use crate::domain::grid::GridError;

#[derive(Debug)]
pub enum AppError {
    Validation(String),
    Auth(String),
    NotFound(String),
    Internal(String),
    Conflict(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Self::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Self::Auth(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            Self::Internal(msg) => {
                tracing::error!("internal error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".into(),
                )
            }
            Self::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
        };
        (status, axum::Json(json!({ "error": message }))).into_response()
    }
}

impl From<GridError> for AppError {
    fn from(err: GridError) -> Self {
        Self::Validation(err.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                Self::Conflict("resource already exists".into())
            }
            _ => Self::Internal(err.to_string()),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        Self::Auth(format!("invalid token: {err}"))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::Internal(err.to_string())
    }
}
