//! The day view: aggregated data for the Today screen.

use axum::{Router, routing::get};

use crate::AppState;

mod handler;
mod model;
mod repository;
mod service;

/// Routes of the day view, mounted under `/api/v1`.
pub fn router() -> Router<AppState> {
    Router::new().route("/days/{date}", get(handler::day_view))
}
