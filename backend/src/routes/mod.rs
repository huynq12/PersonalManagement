//! HTTP routes, one submodule per feature.

use axum::{Router, routing::get};

use crate::AppState;

mod health;

/// Routes mounted under the `/api/v1` prefix.
pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(health::health))
}
