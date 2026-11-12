//! To-dos: one-off checklist items that belong to a specific day.

use axum::{
    Router,
    routing::{get, patch},
};

use crate::AppState;

mod handler;
pub(crate) mod model;
pub(crate) mod repository;
mod service;

/// Routes of the to-dos feature, mounted under `/api/v1`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tasks", get(handler::list).post(handler::create))
        .route(
            "/tasks/{id}",
            patch(handler::update).delete(handler::delete),
        )
}
