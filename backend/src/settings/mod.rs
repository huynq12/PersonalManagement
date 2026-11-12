//! Pomodoro settings: one row per user, created together with the account.

use axum::{Router, routing::get};

use crate::AppState;

mod handler;
mod model;
mod repository;

/// Routes of the settings feature, mounted under `/api/v1`.
pub fn router() -> Router<AppState> {
    Router::new().route("/settings", get(handler::get).put(handler::replace))
}
