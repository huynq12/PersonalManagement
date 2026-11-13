//! User settings: Pomodoro lengths, work settings (hourly wage, base hours), and the monthly
//! budget.

use axum::{Router, routing::get};

use crate::AppState;

mod handler;
mod model;
mod repository;
mod service;

/// Routes of the settings feature, mounted under `/api/v1`.
pub fn router() -> Router<AppState> {
    Router::new().route("/settings", get(handler::get).patch(handler::update))
}
