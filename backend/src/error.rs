//! The API error type and its JSON representation.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

/// Every failure a handler can return.
///
/// Serialized as `{ "error": { "code": "...", "message": "..." } }`. `code` is a stable
/// identifier the frontend translates for users; `message` is English and developer-facing.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// The route or resource does not exist, or belongs to another user.
    #[error("resource not found")]
    NotFound,
    /// A dependency such as the database is unreachable.
    #[error("service temporarily unavailable")]
    ServiceUnavailable,
    /// An unexpected database failure.
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    /// Any other unexpected failure.
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    fn status_and_code(&self) -> (StatusCode, &'static str) {
        match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            Self::ServiceUnavailable => (StatusCode::SERVICE_UNAVAILABLE, "service_unavailable"),
            Self::Database(_) | Self::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error")
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = self.status_and_code();

        // Internal details are logged here, once, and never sent to the client.
        let message = if status == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!(error = ?self, "unhandled error");
            "internal server error".to_owned()
        } else {
            self.to_string()
        };

        let body = Json(json!({ "error": { "code": code, "message": message } }));
        (status, body).into_response()
    }
}

/// Result alias used by handlers.
pub type AppResult<T> = Result<T, AppError>;
