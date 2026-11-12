//! Study sessions: the focus periods of the Pomodoro timer.

use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::AppState;

mod handler;
mod model;
mod repository;
mod service;

/// Routes of the study sessions feature, mounted under `/api/v1`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/study-sessions", get(handler::list).post(handler::start))
        .route("/study-sessions/current", get(handler::current))
        .route("/study-sessions/{id}", delete(handler::delete))
        .route("/study-sessions/{id}/complete", post(handler::complete))
        .route("/study-sessions/{id}/cancel", post(handler::cancel))
}
