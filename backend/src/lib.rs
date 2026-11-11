//! Backend for the personal management app: an Axum HTTP API backed by PostgreSQL.

use anyhow::Context;
use axum::{
    Router,
    http::{HeaderValue, Method, header},
};
use sqlx::PgPool;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub mod config;
pub mod error;
pub mod routes;

/// State shared by every handler.
#[derive(Debug, Clone)]
pub struct AppState {
    /// PostgreSQL connection pool.
    pub db: PgPool,
}

/// Builds the whole application. Kept out of `main` so integration tests can call it directly.
///
/// # Errors
///
/// Returns an error if `cors_origin` is not a valid header value.
pub fn app(state: AppState, cors_origin: &str) -> anyhow::Result<Router> {
    let origin: HeaderValue = cors_origin
        .parse()
        .context("CORS_ORIGIN is not a valid origin")?;

    // Cookies (the refresh token) are sent cross-origin, so the origin and headers must be
    // listed explicitly; wildcards are not allowed together with credentials.
    let cors = CorsLayer::new()
        .allow_origin(origin)
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    Ok(Router::new()
        .nest("/api/v1", routes::router())
        .fallback(|| async { error::AppError::NotFound })
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state))
}
