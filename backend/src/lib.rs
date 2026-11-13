//! Backend for the personal management app: an Axum HTTP API backed by PostgreSQL.

use std::sync::Arc;

use anyhow::Context;
use axum::{
    Router,
    http::{HeaderValue, Method, header},
};
use sqlx::PgPool;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{auth::GoogleClient, config::Config};

pub mod auth;
pub mod config;
pub mod days;
pub mod error;
pub mod habits;
pub mod health;
pub mod study_sessions;
pub mod subjects;
pub mod tasks;
pub mod validation;

/// State shared by every handler. Cloning is cheap: the fields are reference counted.
#[derive(Debug, Clone)]
pub struct AppState {
    /// PostgreSQL connection pool.
    pub db: PgPool,
    /// Application settings.
    pub config: Arc<Config>,
    /// Google OAuth client; `None` when Google sign-in is not configured.
    pub google: Option<Arc<GoogleClient>>,
}

impl AppState {
    /// Bundles the shared resources handlers need.
    ///
    /// # Errors
    ///
    /// Returns an error if the Google OAuth client cannot be initialized.
    pub fn new(db: PgPool, config: Config) -> anyhow::Result<Self> {
        let google = config
            .google
            .clone()
            .map(GoogleClient::new)
            .transpose()?
            .map(Arc::new);

        Ok(Self {
            db,
            config: Arc::new(config),
            google,
        })
    }
}

/// Builds the whole application. Kept out of `main` so integration tests can call it directly.
///
/// # Errors
///
/// Returns an error if the configured CORS origin is not a valid header value.
pub fn app(state: AppState) -> anyhow::Result<Router> {
    let origin: HeaderValue = state
        .config
        .cors_origin
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

    let api = Router::new()
        .merge(health::router())
        .merge(auth::router())
        .merge(habits::router())
        .merge(subjects::router())
        .merge(study_sessions::router())
        .merge(tasks::router())
        .merge(days::router());

    Ok(Router::new()
        .nest("/api/v1", api)
        .fallback(|| async { error::AppError::NotFound })
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state))
}
