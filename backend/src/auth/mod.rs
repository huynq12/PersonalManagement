//! Accounts and sign-in: registration, login, refresh token rotation, logout, Google OAuth,
//! and `/me`.

use axum::{
    Router,
    routing::{get, post},
};

use crate::AppState;

mod extractor;
mod google;
mod handler;
mod model;
mod password;
mod repository;
mod service;
mod token;

pub use extractor::AuthUser;
pub use google::GoogleClient;

/// Routes of the auth feature, mounted under `/api/v1`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(handler::register))
        .route("/auth/login", post(handler::login))
        .route("/auth/refresh", post(handler::refresh))
        .route("/auth/logout", post(handler::logout))
        .route("/auth/google", get(handler::google_start))
        .route("/auth/google/callback", get(handler::google_callback))
        .route("/auth/google/link", post(handler::google_link))
        .route("/me", get(handler::me))
}
