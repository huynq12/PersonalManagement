//! Habits: recurring checklist items with a weekday schedule, completions, and streaks.

use axum::{
    Router,
    routing::{get, patch, put},
};

use crate::AppState;

mod handler;
mod model;
mod repository;
mod service;

/// Routes of the habits feature, mounted under `/api/v1`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/habits", get(handler::list).post(handler::create))
        .route(
            "/habits/{id}",
            patch(handler::update).delete(handler::delete),
        )
        .route(
            "/habits/{id}/completions/{date}",
            put(handler::complete).delete(handler::uncomplete),
        )
}
