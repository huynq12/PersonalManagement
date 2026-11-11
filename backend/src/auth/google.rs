//! The Google side of sign-in: the authorization code flow with PKCE, written by hand.
//!
//! 1. [`GoogleClient::start`] builds the URL the browser is sent to and a signed "flow token"
//!    holding the values that must survive the round trip (`state`, `nonce`, PKCE verifier).
//!    The handler stores the flow token in a short-lived cookie.
//! 2. Google redirects the browser back with a `code`. [`GoogleClient::complete`] checks the
//!    `state`, exchanges the code (proving possession of the PKCE verifier), and verifies the
//!    returned ID token against Google's published keys.

use std::time::Duration;

use anyhow::Context;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation,
    jwk::{Jwk, JwkSet},
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::model::Locale;
use crate::{
    config::{GoogleConfig, Secret},
    error::{AppError, AppResult},
};

/// How long the user has to finish signing in at Google: 10 minutes.
pub(super) const FLOW_TTL_SECONDS: i64 = 10 * 60;

/// Marks a JWT as a flow token, so an access token cannot be replayed as one.
const FLOW_AUDIENCE: &str = "oauth_flow";
const SCOPES: &str = "openid email profile";
/// Both spellings appear in Google ID tokens.
const ISSUERS: [&str; 2] = ["https://accounts.google.com", "accounts.google.com"];
const RANDOM_BYTES: usize = 32;
const HTTP_TIMEOUT: Duration = Duration::from_secs(10);

/// Values that must survive the browser's round trip to Google. Travels in a signed cookie.
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct Flow {
    /// Random value echoed back by Google; proves the callback belongs to this browser.
    state: String,
    /// Random value Google copies into the ID token; proves the token belongs to this flow.
    nonce: String,
    /// Secret whose hash was sent to Google; proves the code is exchanged by whoever started.
    pkce_verifier: String,
    /// The signed-in user when the flow links Google to an existing account.
    pub link_user_id: Option<Uuid>,
    /// Interface language to give an account created by this flow.
    pub locale: Option<Locale>,
    aud: String,
    exp: i64,
}

/// What [`GoogleClient::start`] produces.
pub(super) struct StartedFlow {
    /// The Google URL to send the browser to.
    pub authorization_url: String,
    /// The signed flow token to store in a cookie.
    pub flow_token: String,
}

/// Query parameters Google adds when it redirects the browser back.
#[derive(Debug, Deserialize)]
pub(super) struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    /// Set instead of `code` when the user cancels or Google refuses.
    error: Option<String>,
}

/// The identity asserted by a verified ID token.
#[derive(Debug)]
pub(super) struct GoogleProfile {
    /// Google's stable identifier for the account. Emails can change; this cannot.
    pub subject: String,
    pub email: String,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    id_token: String,
}

#[derive(Debug, Deserialize)]
struct IdTokenClaims {
    sub: String,
    email: Option<String>,
    #[serde(default)]
    email_verified: bool,
    name: Option<String>,
    nonce: Option<String>,
}

/// Talks to Google's OAuth endpoints and caches the keys that sign ID tokens.
#[derive(Debug)]
pub struct GoogleClient {
    config: GoogleConfig,
    http: reqwest::Client,
    /// Google's signing keys. Refetched when a token names a key that is not cached, which
    /// is how key rotation is picked up.
    jwks: RwLock<Option<JwkSet>>,
}

impl GoogleClient {
    /// Creates a client for the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be initialized.
    pub fn new(config: GoogleConfig) -> anyhow::Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(HTTP_TIMEOUT)
            // The endpoints never redirect; following one could leak the client secret.
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .context("failed to build the HTTP client for Google")?;

        Ok(Self {
            config,
            http,
            jwks: RwLock::new(None),
        })
    }

    /// Starts a flow. `link_user_id` is the signed-in user when linking, `None` when signing in.
    ///
    /// # Errors
    ///
    /// Returns an error if the configured authorization URL is malformed or signing fails.
    pub(super) fn start(
        &self,
        jwt_secret: &Secret,
        link_user_id: Option<Uuid>,
        locale: Option<Locale>,
    ) -> anyhow::Result<StartedFlow> {
        let state = random_token();
        let nonce = random_token();
        let pkce_verifier = random_token();
        let code_challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(pkce_verifier.as_bytes()));

        let authorization_url = Url::parse_with_params(
            &self.config.auth_url,
            [
                ("response_type", "code"),
                ("client_id", self.config.client_id.as_str()),
                ("redirect_uri", self.config.redirect_url.as_str()),
                ("scope", SCOPES),
                ("state", state.as_str()),
                ("nonce", nonce.as_str()),
                ("code_challenge", code_challenge.as_str()),
                ("code_challenge_method", "S256"),
            ],
        )
        .context("GOOGLE auth URL is not a valid URL")?
        .into();

        let flow = Flow {
            state,
            nonce,
            pkce_verifier,
            link_user_id,
            locale,
            aud: FLOW_AUDIENCE.to_owned(),
            exp: Utc::now().timestamp() + FLOW_TTL_SECONDS,
        };
        let flow_token = jsonwebtoken::encode(
            &Header::new(Algorithm::HS256),
            &flow,
            &EncodingKey::from_secret(jwt_secret.expose().as_bytes()),
        )
        .context("failed to sign the OAuth flow token")?;

        Ok(StartedFlow {
            authorization_url,
            flow_token,
        })
    }

    /// Finishes a flow and returns the verified Google identity.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::GoogleSignInFailed`] if Google reported an error, the `state` does
    /// not match, the code is rejected, or the ID token is invalid, and
    /// [`AppError::GoogleEmailUnverified`] if Google has not verified the account's email.
    pub(super) async fn complete(
        &self,
        flow: &Flow,
        query: CallbackQuery,
    ) -> AppResult<GoogleProfile> {
        if let Some(error) = query.error {
            tracing::info!(%error, "google returned an error instead of a code");
            return Err(AppError::GoogleSignInFailed);
        }
        let (Some(code), Some(state)) = (query.code, query.state) else {
            return Err(AppError::GoogleSignInFailed);
        };
        if state != flow.state {
            tracing::warn!("google callback state does not match the flow cookie");
            return Err(AppError::GoogleSignInFailed);
        }

        let id_token = self
            .exchange_code(&code, &flow.pkce_verifier)
            .await
            .map_err(sign_in_failed("failed to exchange the authorization code"))?;
        let claims = self
            .verify_id_token(&id_token)
            .await
            .map_err(sign_in_failed("failed to verify the google ID token"))?;

        if claims.nonce.as_deref() != Some(flow.nonce.as_str()) {
            tracing::warn!("google ID token nonce does not match the flow cookie");
            return Err(AppError::GoogleSignInFailed);
        }
        let Some(email) = claims.email else {
            tracing::warn!("google ID token has no email claim");
            return Err(AppError::GoogleSignInFailed);
        };
        if !claims.email_verified {
            return Err(AppError::GoogleEmailUnverified);
        }

        Ok(GoogleProfile {
            subject: claims.sub,
            email,
            name: claims.name,
        })
    }

    /// Exchanges the authorization code for an ID token.
    async fn exchange_code(&self, code: &str, pkce_verifier: &str) -> anyhow::Result<String> {
        let response = self
            .http
            .post(&self.config.token_url)
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("client_id", self.config.client_id.as_str()),
                ("client_secret", self.config.client_secret.expose()),
                ("redirect_uri", self.config.redirect_url.as_str()),
                ("code_verifier", pkce_verifier),
            ])
            .send()
            .await
            .context("token request failed")?
            .error_for_status()
            .context("token endpoint rejected the request")?;

        let token: TokenResponse = response
            .json()
            .await
            .context("token response is not the expected JSON")?;
        Ok(token.id_token)
    }

    /// Verifies the ID token's signature, issuer, audience, and expiry.
    async fn verify_id_token(&self, id_token: &str) -> anyhow::Result<IdTokenClaims> {
        let header = jsonwebtoken::decode_header(id_token).context("ID token is malformed")?;
        let key_id = header.kid.context("ID token header has no key ID")?;
        let jwk = self.signing_key(&key_id).await?;
        let key = DecodingKey::from_jwk(&jwk).context("google signing key is not usable")?;

        // Pinning the algorithm stops a token from choosing a weaker one for itself.
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[self.config.client_id.as_str()]);
        validation.set_issuer(&ISSUERS);
        validation.set_required_spec_claims(&["exp", "aud", "iss", "sub"]);

        let data = jsonwebtoken::decode::<IdTokenClaims>(id_token, &key, &validation)
            .context("ID token failed validation")?;
        Ok(data.claims)
    }

    /// Returns the key with the given ID, fetching Google's key set if it is not cached.
    async fn signing_key(&self, key_id: &str) -> anyhow::Result<Jwk> {
        if let Some(jwk) = self
            .jwks
            .read()
            .await
            .as_ref()
            .and_then(|keys| keys.find(key_id))
        {
            return Ok(jwk.clone());
        }

        let keys: JwkSet = self
            .http
            .get(&self.config.jwks_url)
            .send()
            .await
            .context("JWKS request failed")?
            .error_for_status()
            .context("JWKS endpoint rejected the request")?
            .json()
            .await
            .context("JWKS response is not the expected JSON")?;

        let jwk = keys.find(key_id).cloned();
        *self.jwks.write().await = Some(keys);
        jwk.context("ID token is signed with a key google does not publish")
    }
}

/// Decodes and verifies a flow token. Returns `None` if it is missing, tampered, or expired.
pub(super) fn decode_flow(jwt_secret: &Secret, flow_token: &str) -> Option<Flow> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&[FLOW_AUDIENCE]);
    validation.set_required_spec_claims(&["exp", "aud"]);

    jsonwebtoken::decode::<Flow>(
        flow_token,
        &DecodingKey::from_secret(jwt_secret.expose().as_bytes()),
        &validation,
    )
    .ok()
    .map(|data| data.claims)
}

/// 256 bits of cryptographically secure randomness, URL-safe (43 characters).
fn random_token() -> String {
    let bytes: [u8; RANDOM_BYTES] = rand::random();
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Logs why a step failed and reports it to the user as a generic sign-in failure.
fn sign_in_failed(step: &'static str) -> impl FnOnce(anyhow::Error) -> AppError {
    move |error| {
        tracing::warn!(error = ?error, "{step}");
        AppError::GoogleSignInFailed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secret() -> Secret {
        Secret::new("unit-test-secret-unit-test-secret-unit")
    }

    fn client() -> GoogleClient {
        GoogleClient::new(GoogleConfig::new(
            "client-id",
            Secret::new("client-secret"),
            "http://localhost:8080/api/v1/auth/google/callback",
        ))
        .unwrap()
    }

    #[test]
    fn flow_token_round_trips_through_decode_flow() {
        let user_id = Uuid::new_v4();

        let started = client()
            .start(&secret(), Some(user_id), Some(Locale::Vi))
            .unwrap();
        let flow = decode_flow(&secret(), &started.flow_token).unwrap();

        assert_eq!(flow.link_user_id, Some(user_id));
        assert_eq!(flow.locale, Some(Locale::Vi));
        assert!(started.authorization_url.contains(&flow.state));
    }

    #[test]
    fn flow_token_signed_with_another_secret_is_rejected() {
        let started = client().start(&secret(), None, None).unwrap();

        assert!(decode_flow(&Secret::new("y".repeat(32)), &started.flow_token).is_none());
    }

    #[test]
    fn authorization_url_carries_the_challenge_but_never_the_verifier() {
        let started = client().start(&secret(), None, None).unwrap();
        let flow = decode_flow(&secret(), &started.flow_token).unwrap();

        assert!(!started.authorization_url.contains(&flow.pkce_verifier));
        assert!(!started.authorization_url.contains("client-secret"));
    }
}
