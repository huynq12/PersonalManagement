//! The day view: aggregated data for the Today screen, and the day mode.

use axum::{
    Router,
    routing::{get, put},
};

use crate::AppState;

mod handler;
mod model;
mod repository;
mod service;

/// Routes of the day view, mounted under `/api/v1`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/days/{date}", get(handler::day_view))
        .route("/days/{date}/mode", put(handler::set_mode))
}
