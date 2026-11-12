//! Pomodoro settings: the lengths of focus sessions and breaks. One row per user.

use axum::{Router, routing::get};

use crate::AppState;

mod handler;
mod model;
mod repository;
mod service;

/// Routes of the settings feature, mounted under `/api/v1`.
pub fn router() -> Router<AppState> {
    Router::new().route("/settings", get(handler::get).put(handler::update))
}
