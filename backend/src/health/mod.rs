//! Liveness check for the server and its database connection.

use axum::{Router, routing::get};

use crate::AppState;

mod handler;

/// Routes of the health feature, mounted under `/api/v1`.
pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(handler::health))
}
