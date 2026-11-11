//! Runtime configuration read from environment variables.

use std::env;

use anyhow::Context;

const DEFAULT_PORT: u16 = 8080;
const DEFAULT_CORS_ORIGIN: &str = "http://localhost:5173";

/// Application settings. Locally these come from the `.env` file.
#[derive(Debug, Clone)]
pub struct Config {
    /// PostgreSQL connection string.
    pub database_url: String,
    /// TCP port the HTTP server listens on.
    pub port: u16,
    /// The single browser origin allowed to call the API.
    pub cors_origin: String,
}

impl Config {
    /// Reads the configuration from the process environment.
    ///
    /// # Errors
    ///
    /// Returns an error if `DATABASE_URL` is missing or `PORT` is not a valid port number.
    pub fn from_env() -> anyhow::Result<Self> {
        let database_url = env::var("DATABASE_URL").context("DATABASE_URL is not set")?;
        let port = match env::var("PORT") {
            Ok(value) => value.parse().context("PORT must be a port number")?,
            Err(_) => DEFAULT_PORT,
        };
        let cors_origin = env::var("CORS_ORIGIN").unwrap_or_else(|_| DEFAULT_CORS_ORIGIN.into());

        Ok(Self {
            database_url,
            port,
            cors_origin,
        })
    }
}
