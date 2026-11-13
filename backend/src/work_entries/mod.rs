//! Work log: the hours worked per day, the pay computed from them, and monthly totals.

use axum::{Router, routing::get};

use crate::AppState;

mod handler;
mod model;
mod repository;
mod service;

/// Routes of the work log feature, mounted under `/api/v1`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/work-entries", get(handler::list))
        .route(
            "/work-entries/{date}",
            axum::routing::put(handler::put).delete(handler::delete),
        )
}
