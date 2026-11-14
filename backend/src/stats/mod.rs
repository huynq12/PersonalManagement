//! Weekly statistics: study time, checklist rates, day modes, and habit streaks.

use axum::{Router, routing::get};

use crate::AppState;

mod handler;
mod model;
mod repository;
mod service;

/// Routes of the statistics feature, mounted under `/api/v1`.
pub fn router() -> Router<AppState> {
    Router::new().route("/stats/weekly", get(handler::weekly))
}
