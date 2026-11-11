//! Runtime configuration read from environment variables.

use std::{env, fmt};

use anyhow::{Context, bail};

const DEFAULT_PORT: u16 = 8080;
const DEFAULT_CORS_ORIGIN: &str = "http://localhost:5173";
const MIN_JWT_SECRET_BYTES: usize = 32;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_JWKS_URL: &str = "https://www.googleapis.com/oauth2/v3/certs";

/// A sensitive string whose `Debug` output is redacted, so it cannot leak into logs.
#[derive(Clone)]
pub struct Secret(String);

impl Secret {
    /// Wraps a sensitive value.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the wrapped value. Call this only where the secret is actually used.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(<redacted>)")
    }
}

/// Application settings. Locally these come from the `.env` file.
#[derive(Debug, Clone)]
pub struct Config {
    /// PostgreSQL connection string used by the running app.
    pub database_url: Secret,
    /// TCP port the HTTP server listens on.
    pub port: u16,
    /// Origin of the frontend: the single browser origin allowed to call the API, and the
    /// base URL for redirects after Google sign-in.
    pub cors_origin: String,
    /// Symmetric key that signs access tokens and the OAuth flow cookie (HS256).
    pub jwt_secret: Secret,
    /// Whether cookies carry the `Secure` attribute. Must be `true` behind HTTPS.
    pub cookie_secure: bool,
    /// Google OAuth settings; `None` disables the Google routes.
    pub google: Option<GoogleConfig>,
}

/// Settings for the Google OAuth client.
#[derive(Debug, Clone)]
pub struct GoogleConfig {
    /// OAuth client ID from the Google Cloud console.
    pub client_id: String,
    /// OAuth client secret from the Google Cloud console.
    pub client_secret: Secret,
    /// The callback URL registered with Google; must point at `/api/v1/auth/google/callback`.
    pub redirect_url: String,
    /// Authorization endpoint the browser is sent to.
    pub auth_url: String,
    /// Token endpoint where the authorization code is exchanged.
    pub token_url: String,
    /// JWKS endpoint holding the keys that sign ID tokens.
    pub jwks_url: String,
}

impl GoogleConfig {
    /// Builds a configuration that talks to the real Google endpoints.
    pub fn new(
        client_id: impl Into<String>,
        client_secret: Secret,
        redirect_url: impl Into<String>,
    ) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret,
            redirect_url: redirect_url.into(),
            auth_url: GOOGLE_AUTH_URL.to_owned(),
            token_url: GOOGLE_TOKEN_URL.to_owned(),
            jwks_url: GOOGLE_JWKS_URL.to_owned(),
        }
    }
}

impl Config {
    /// Reads the configuration from the process environment.
    ///
    /// The app connects to `APP_DATABASE_URL` and falls back to `DATABASE_URL`. Tests and
    /// SQLx tooling always read `DATABASE_URL`, so the two can point at different databases.
    ///
    /// # Errors
    ///
    /// Returns an error if a required variable is missing or a value is malformed.
    pub fn from_env() -> anyhow::Result<Self> {
        let database_url = optional("APP_DATABASE_URL")
            .or_else(|| optional("DATABASE_URL"))
            .context("APP_DATABASE_URL or DATABASE_URL must be set")?;
        let port = match optional("PORT") {
            Some(value) => value.parse().context("PORT must be a port number")?,
            None => DEFAULT_PORT,
        };
        let cors_origin = optional("CORS_ORIGIN").unwrap_or_else(|| DEFAULT_CORS_ORIGIN.into());

        let jwt_secret = optional("JWT_SECRET").context("JWT_SECRET is not set")?;
        if jwt_secret.len() < MIN_JWT_SECRET_BYTES {
            bail!("JWT_SECRET must be at least {MIN_JWT_SECRET_BYTES} bytes long");
        }

        let cookie_secure = match optional("COOKIE_SECURE") {
            Some(value) => value
                .parse()
                .context("COOKIE_SECURE must be true or false")?,
            None => false,
        };

        let google = match optional("GOOGLE_CLIENT_ID") {
            Some(client_id) => {
                let client_secret = optional("GOOGLE_CLIENT_SECRET")
                    .context("GOOGLE_CLIENT_SECRET is required when GOOGLE_CLIENT_ID is set")?;
                let redirect_url = optional("GOOGLE_REDIRECT_URL")
                    .context("GOOGLE_REDIRECT_URL is required when GOOGLE_CLIENT_ID is set")?;
                Some(GoogleConfig::new(
                    client_id,
                    Secret::new(client_secret),
                    redirect_url,
                ))
            }
            None => None,
        };

        Ok(Self {
            database_url: Secret::new(database_url),
            port,
            cors_origin,
            jwt_secret: Secret::new(jwt_secret),
            cookie_secure,
            google,
        })
    }
}

/// Reads a variable, treating an empty value the same as a missing one.
fn optional(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_debug_output_is_redacted() {
        let secret = Secret::new("hunter2");

        assert_eq!(format!("{secret:?}"), "Secret(<redacted>)");
        assert_eq!(secret.expose(), "hunter2");
    }
}
