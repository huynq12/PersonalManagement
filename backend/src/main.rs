//! Server entry point: loads configuration, connects to the database, and serves the API.

use std::net::{Ipv4Addr, SocketAddr};

use anyhow::Context;
use backend::{AppState, app, config::Config};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

const MAX_DB_CONNECTIONS: u32 = 5;
const DEFAULT_LOG_FILTER: &str = "backend=debug,tower_http=debug,info";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // A missing `.env` file is fine: hosting platforms set the variables directly.
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_FILTER)),
        )
        .init();

    let config = Config::from_env()?;

    let db = PgPoolOptions::new()
        .max_connections(MAX_DB_CONNECTIONS)
        .connect(&config.database_url)
        .await
        .context("failed to connect to the database")?;

    sqlx::migrate!()
        .run(&db)
        .await
        .context("failed to run database migrations")?;

    let app = app(AppState { db }, &config.cors_origin)?;

    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, config.port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;
    tracing::info!(%addr, "server listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    Ok(())
}

/// Resolves on Ctrl+C or, on Unix, SIGTERM (what container platforms send on deploy).
async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(%error, "failed to listen for Ctrl+C");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};

        match signal(SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
            }
            Err(error) => {
                tracing::error!(%error, "failed to listen for SIGTERM");
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }

    tracing::info!("shutdown signal received, stopping server");
}
