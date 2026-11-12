//! Subjects: the named, colored topics a user studies in Pomodoro sessions.

use axum::{
    Router,
    routing::{get, patch},
};

use crate::AppState;

mod handler;
mod model;
pub(crate) mod repository;
mod service;

/// Routes of the subjects feature, mounted under `/api/v1`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/subjects", get(handler::list).post(handler::create))
        .route(
            "/subjects/{id}",
            patch(handler::update).delete(handler::delete),
        )
}
