//! Integration tests for Google sign-in and account linking.
//!
//! Google is replaced by a local mock server that serves the token endpoint and the JWKS. The
//! ID tokens it returns are signed with a test-only RSA key stored under `tests/fixtures/`.

// Test helpers are not `#[test]` functions, so the clippy.toml test exemption does not reach them.
#![allow(clippy::unwrap_used)]

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use axum::{
    Form, Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use personal_management_api::config::{Config, GoogleConfig, Secret};
use reqwest::Url;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::common::{FRONTEND_ORIGIN, TestRequest, TestResponse};

mod common;

const TEST_KEY_PEM: &str = include_str!("fixtures/google_test_key.pem");
const TEST_JWKS: &str = include_str!("fixtures/google_test_jwks.json");
const TEST_KEY_ID: &str = "test-key-1";

const CLIENT_ID: &str = "test-client-id";
const REDIRECT_URL: &str = "http://localhost:8080/api/v1/auth/google/callback";
const AUTH_URL: &str = "https://accounts.google.test/authorize";

const GOOGLE_EMAIL: &str = "ada@example.com";
const GOOGLE_SUBJECT: &str = "google-subject-1";
const PASSWORD: &str = "correct horse battery";

const FLOW_COOKIE: &str = "oauth_flow";
const REFRESH_COOKIE: &str = "refresh_token";

// ---------------------------------------------------------------------------------------------
// Mock Google
// ---------------------------------------------------------------------------------------------

/// What the mock token endpoint answers, and what it was asked.
struct MockState {
    /// Claims of the ID token to return.
    claims: Value,
    /// `kid` written into the ID token header.
    key_id: String,
    /// Status of the token endpoint; anything but 200 simulates a rejected code.
    token_status: StatusCode,
    /// The form of the last token request.
    last_token_request: Option<HashMap<String, String>>,
}

#[derive(Clone)]
struct MockGoogle {
    base_url: String,
    state: Arc<Mutex<MockState>>,
}

impl MockGoogle {
    /// Starts the mock on a random local port.
    async fn start() -> Self {
        let state = Arc::new(Mutex::new(MockState {
            claims: Value::Null,
            key_id: TEST_KEY_ID.to_owned(),
            token_status: StatusCode::OK,
            last_token_request: None,
        }));

        let router = Router::new()
            .route("/token", post(mock_token))
            .route("/certs", get(mock_certs))
            .with_state(Arc::clone(&state));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });

        Self { base_url, state }
    }

    fn config(&self) -> Config {
        let mut config = common::config();
        config.google = Some(GoogleConfig {
            client_id: CLIENT_ID.to_owned(),
            client_secret: Secret::new("test-client-secret"),
            redirect_url: REDIRECT_URL.to_owned(),
            auth_url: AUTH_URL.to_owned(),
            token_url: format!("{}/token", self.base_url),
            jwks_url: format!("{}/certs", self.base_url),
        });
        config
    }

    fn app(&self, db: PgPool) -> Router {
        common::app_with(db, self.config())
    }

    /// Sets the ID token claims the next token request returns.
    fn respond_with(&self, claims: Value) {
        self.state.lock().unwrap().claims = claims;
    }

    fn update(&self, change: impl FnOnce(&mut MockState)) {
        change(&mut self.state.lock().unwrap());
    }

    fn last_token_request(&self) -> HashMap<String, String> {
        self.state
            .lock()
            .unwrap()
            .last_token_request
            .clone()
            .unwrap()
    }
}

async fn mock_token(
    State(state): State<Arc<Mutex<MockState>>>,
    Form(form): Form<HashMap<String, String>>,
) -> Response {
    let mut state = state.lock().unwrap();
    state.last_token_request = Some(form);
    if state.token_status != StatusCode::OK {
        return (
            state.token_status,
            Json(json!({ "error": "invalid_grant" })),
        )
            .into_response();
    }

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(state.key_id.clone());
    let key = EncodingKey::from_rsa_pem(TEST_KEY_PEM.as_bytes()).unwrap();
    let id_token = jsonwebtoken::encode(&header, &state.claims, &key).unwrap();

    Json(json!({
        "access_token": "mock-google-access-token",
        "token_type": "Bearer",
        "expires_in": 3600,
        "id_token": id_token,
    }))
    .into_response()
}

async fn mock_certs() -> Response {
    ([("content-type", "application/json")], TEST_JWKS).into_response()
}

// ---------------------------------------------------------------------------------------------
// Flow helpers
// ---------------------------------------------------------------------------------------------

/// The values the app generated when it started a Google flow.
struct StartedFlow {
    state: String,
    nonce: String,
    code_challenge: String,
    flow_cookie: String,
}

impl StartedFlow {
    /// Reads the flow from the authorization URL and the response that carried the cookie.
    fn parse(authorization_url: &str, response: &TestResponse) -> Self {
        let params = query_params(authorization_url);
        Self {
            state: params["state"].clone(),
            nonce: params["nonce"].clone(),
            code_challenge: params["code_challenge"].clone(),
            flow_cookie: response.cookie_value(FLOW_COOKIE).unwrap().to_owned(),
        }
    }
}

fn query_params(url: &str) -> HashMap<String, String> {
    Url::parse(url)
        .unwrap()
        .query_pairs()
        .map(|(name, value)| (name.into_owned(), value.into_owned()))
        .collect()
}

/// Valid ID token claims for the default Google account.
fn claims(nonce: &str) -> Value {
    let now = chrono::Utc::now().timestamp();
    json!({
        "iss": "https://accounts.google.com",
        "aud": CLIENT_ID,
        "sub": GOOGLE_SUBJECT,
        "email": GOOGLE_EMAIL,
        "email_verified": true,
        "name": "Ada Lovelace",
        "nonce": nonce,
        "iat": now,
        "exp": now + 3600,
    })
}

/// Starts a sign-in flow: `GET /auth/google`.
async fn start_sign_in(app: &Router) -> StartedFlow {
    let response = TestRequest::get("/api/v1/auth/google").send(app).await;
    assert_eq!(response.status, StatusCode::SEE_OTHER);
    StartedFlow::parse(response.location(), &response)
}

/// Starts a linking flow for the signed-in user: `POST /auth/google/link`.
async fn start_link(app: &Router, access_token: &str) -> StartedFlow {
    let response = TestRequest::post("/api/v1/auth/google/link")
        .bearer(access_token)
        .send(app)
        .await;
    assert_eq!(response.status, StatusCode::OK);
    let url = response.body["authorization_url"].as_str().unwrap();
    StartedFlow::parse(url, &response)
}

/// Simulates Google redirecting the browser back with a code.
async fn callback(app: &Router, flow: &StartedFlow) -> TestResponse {
    callback_with(app, &flow.flow_cookie, &flow.state).await
}

async fn callback_with(app: &Router, flow_cookie: &str, state: &str) -> TestResponse {
    TestRequest::get(&format!(
        "/api/v1/auth/google/callback?code=test-code&state={state}"
    ))
    .cookie(FLOW_COOKIE, flow_cookie)
    .send(app)
    .await
}

/// Runs a whole successful sign-in for the given claims and returns the callback response.
async fn sign_in_with(
    app: &Router,
    google: &MockGoogle,
    edit: impl FnOnce(&mut Value),
) -> TestResponse {
    let flow = start_sign_in(app).await;
    let mut id_token_claims = claims(&flow.nonce);
    edit(&mut id_token_claims);
    google.respond_with(id_token_claims);
    callback(app, &flow).await
}

async fn sign_in(app: &Router, google: &MockGoogle) -> TestResponse {
    sign_in_with(app, google, |_| {}).await
}

/// Registers a password account and returns its access token.
async fn register_and_login(app: &Router, email: &str) -> String {
    let registered = TestRequest::post("/api/v1/auth/register")
        .json(&json!({ "email": email, "password": PASSWORD }))
        .send(app)
        .await;
    assert_eq!(registered.status, StatusCode::CREATED);
    let session = TestRequest::post("/api/v1/auth/login")
        .json(&json!({ "email": email, "password": PASSWORD }))
        .send(app)
        .await;
    session.body["access_token"].as_str().unwrap().to_owned()
}

fn assert_sign_in_failed(response: &TestResponse, code: &str) {
    assert_eq!(response.status, StatusCode::SEE_OTHER);
    assert_eq!(
        response.location(),
        format!("{FRONTEND_ORIGIN}/login?error={code}")
    );
    assert!(response.set_cookie(REFRESH_COOKIE).is_none());
}

async fn count(db: &PgPool, table: &str) -> i64 {
    sqlx::query_scalar(&format!("SELECT count(*) FROM {table}"))
        .fetch_one(db)
        .await
        .unwrap()
}

// ---------------------------------------------------------------------------------------------
// Starting the flow
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn start_redirects_to_google_with_the_required_parameters(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db);

    let response = TestRequest::get("/api/v1/auth/google").send(&app).await;

    assert_eq!(response.status, StatusCode::SEE_OTHER);
    let location = response.location();
    assert!(location.starts_with(AUTH_URL));
    let params = query_params(location);
    assert_eq!(params["response_type"], "code");
    assert_eq!(params["client_id"], CLIENT_ID);
    assert_eq!(params["redirect_uri"], REDIRECT_URL);
    assert_eq!(params["scope"], "openid email profile");
    assert_eq!(params["code_challenge_method"], "S256");
    assert!(params["state"].len() >= 32);
    assert!(params["nonce"].len() >= 32);
    assert!(params["code_challenge"].len() >= 43);
    assert!(!location.contains("test-client-secret"));
}

#[sqlx::test]
async fn start_sets_an_http_only_flow_cookie_that_expires_in_10_minutes(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db);

    let response = TestRequest::get("/api/v1/auth/google").send(&app).await;

    let cookie = response.set_cookie(FLOW_COOKIE).unwrap();
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("Max-Age=600"));
    assert!(cookie.contains("Path=/api/v1/auth/google"));
}

#[sqlx::test]
async fn start_generates_fresh_values_for_every_flow(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db);

    let first = start_sign_in(&app).await;
    let second = start_sign_in(&app).await;

    assert_ne!(first.state, second.state);
    assert_ne!(first.nonce, second.nonce);
    assert_ne!(first.code_challenge, second.code_challenge);
}

#[sqlx::test]
async fn google_routes_are_not_found_when_google_is_not_configured(db: PgPool) {
    let app = common::app(db);

    let start = TestRequest::get("/api/v1/auth/google").send(&app).await;
    let callback = TestRequest::get("/api/v1/auth/google/callback?code=x&state=y")
        .send(&app)
        .await;

    assert_eq!(start.status, StatusCode::NOT_FOUND);
    assert_eq!(callback.status, StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------------------------
// Sign-in callback
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn callback_creates_an_account_and_redirects_to_the_frontend_with_a_refresh_cookie(
    db: PgPool,
) {
    let google = MockGoogle::start().await;
    let app = google.app(db.clone());

    let response = sign_in(&app, &google).await;

    assert_eq!(response.status, StatusCode::SEE_OTHER);
    assert_eq!(
        response.location(),
        format!("{FRONTEND_ORIGIN}/auth/callback")
    );
    let refresh_token = response.cookie_value(REFRESH_COOKIE).unwrap();
    let session = TestRequest::post("/api/v1/auth/refresh")
        .cookie(REFRESH_COOKIE, refresh_token)
        .send(&app)
        .await;
    assert_eq!(session.status, StatusCode::OK);
    assert_eq!(session.body["user"]["email"], GOOGLE_EMAIL);
    assert_eq!(session.body["user"]["display_name"], "Ada Lovelace");
    assert_eq!(count(&db, "user_settings").await, 1);
    let password_hash: Option<String> = sqlx::query_scalar("SELECT password_hash FROM users")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(password_hash, None);
}

#[sqlx::test]
async fn callback_never_puts_a_token_in_the_redirect_url(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db);

    let response = sign_in(&app, &google).await;

    assert!(!response.location().contains('?'));
    assert!(!response.location().contains('#'));
}

#[sqlx::test]
async fn callback_clears_the_flow_cookie(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db);

    let response = sign_in(&app, &google).await;

    assert!(
        response
            .set_cookie(FLOW_COOKIE)
            .unwrap()
            .contains("Max-Age=0")
    );
}

#[sqlx::test]
async fn callback_exchanges_the_code_with_the_pkce_verifier_and_client_credentials(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db);
    let flow = start_sign_in(&app).await;
    google.respond_with(claims(&flow.nonce));

    callback(&app, &flow).await;

    let form = google.last_token_request();
    assert_eq!(form["grant_type"], "authorization_code");
    assert_eq!(form["code"], "test-code");
    assert_eq!(form["client_id"], CLIENT_ID);
    assert_eq!(form["client_secret"], "test-client-secret");
    assert_eq!(form["redirect_uri"], REDIRECT_URL);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(form["code_verifier"].as_bytes()));
    assert_eq!(challenge, flow.code_challenge);
}

#[sqlx::test]
async fn callback_signs_in_the_existing_account_on_later_sign_ins(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db.clone());
    sign_in(&app, &google).await;

    let response = sign_in(&app, &google).await;

    assert_eq!(
        response.location(),
        format!("{FRONTEND_ORIGIN}/auth/callback")
    );
    assert_eq!(count(&db, "users").await, 1);
    assert_eq!(count(&db, "oauth_accounts").await, 1);
}

#[sqlx::test]
async fn callback_uses_the_locale_chosen_when_the_flow_started(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db.clone());
    let started = TestRequest::get("/api/v1/auth/google?locale=vi")
        .send(&app)
        .await;
    let flow = StartedFlow::parse(started.location(), &started);
    google.respond_with(claims(&flow.nonce));

    callback(&app, &flow).await;

    let locale: String = sqlx::query_scalar("SELECT locale FROM users")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(locale, "vi");
}

#[sqlx::test]
async fn callback_falls_back_to_the_email_local_part_when_google_sends_no_name(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db.clone());

    sign_in_with(&app, &google, |claims| {
        claims.as_object_mut().unwrap().remove("name");
    })
    .await;

    let display_name: String = sqlx::query_scalar("SELECT display_name FROM users")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(display_name, "ada");
}

#[sqlx::test]
async fn callback_rejects_a_state_that_does_not_match_the_flow_cookie(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db.clone());
    let flow = start_sign_in(&app).await;
    google.respond_with(claims(&flow.nonce));

    let response = callback_with(&app, &flow.flow_cookie, "forged-state").await;

    assert_sign_in_failed(&response, "google_sign_in_failed");
    assert_eq!(count(&db, "users").await, 0);
}

#[sqlx::test]
async fn callback_rejects_a_request_without_the_flow_cookie(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db);
    let flow = start_sign_in(&app).await;
    google.respond_with(claims(&flow.nonce));

    let response = TestRequest::get(&format!(
        "/api/v1/auth/google/callback?code=test-code&state={}",
        flow.state
    ))
    .send(&app)
    .await;

    assert_sign_in_failed(&response, "google_sign_in_failed");
}

#[sqlx::test]
async fn callback_rejects_a_tampered_flow_cookie(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db);
    let flow = start_sign_in(&app).await;
    google.respond_with(claims(&flow.nonce));
    let tampered = format!("{}x", flow.flow_cookie);

    let response = callback_with(&app, &tampered, &flow.state).await;

    assert_sign_in_failed(&response, "google_sign_in_failed");
}

#[sqlx::test]
async fn callback_rejects_an_error_returned_by_google(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db);
    let flow = start_sign_in(&app).await;

    let response = TestRequest::get(&format!(
        "/api/v1/auth/google/callback?error=access_denied&state={}",
        flow.state
    ))
    .cookie(FLOW_COOKIE, &flow.flow_cookie)
    .send(&app)
    .await;

    assert_sign_in_failed(&response, "google_sign_in_failed");
}

#[sqlx::test]
async fn callback_rejects_a_code_that_google_refuses_to_exchange(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db);
    google.update(|state| state.token_status = StatusCode::BAD_REQUEST);

    let response = sign_in(&app, &google).await;

    assert_sign_in_failed(&response, "google_sign_in_failed");
}

#[sqlx::test]
async fn callback_rejects_an_id_token_with_the_wrong_nonce(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db);

    let response = sign_in_with(&app, &google, |claims| {
        claims["nonce"] = json!("nonce-from-another-flow");
    })
    .await;

    assert_sign_in_failed(&response, "google_sign_in_failed");
}

#[sqlx::test]
async fn callback_rejects_an_id_token_issued_for_another_client(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db);

    let response = sign_in_with(&app, &google, |claims| {
        claims["aud"] = json!("someone-elses-client-id");
    })
    .await;

    assert_sign_in_failed(&response, "google_sign_in_failed");
}

#[sqlx::test]
async fn callback_rejects_an_id_token_from_another_issuer(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db);

    let response = sign_in_with(&app, &google, |claims| {
        claims["iss"] = json!("https://evil.example.com");
    })
    .await;

    assert_sign_in_failed(&response, "google_sign_in_failed");
}

#[sqlx::test]
async fn callback_rejects_an_expired_id_token(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db);

    let response = sign_in_with(&app, &google, |claims| {
        claims["exp"] = json!(chrono::Utc::now().timestamp() - 3600);
    })
    .await;

    assert_sign_in_failed(&response, "google_sign_in_failed");
}

#[sqlx::test]
async fn callback_rejects_an_id_token_signed_with_a_key_google_does_not_publish(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db);
    google.update(|state| state.key_id = "unknown-key".to_owned());

    let response = sign_in(&app, &google).await;

    assert_sign_in_failed(&response, "google_sign_in_failed");
}

#[sqlx::test]
async fn callback_rejects_a_google_account_with_an_unverified_email(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db.clone());

    let response = sign_in_with(&app, &google, |claims| {
        claims["email_verified"] = json!(false);
    })
    .await;

    assert_sign_in_failed(&response, "google_email_unverified");
    assert_eq!(count(&db, "users").await, 0);
}

#[sqlx::test]
async fn callback_does_not_merge_into_a_password_account_with_the_same_email(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db.clone());
    register_and_login(&app, "ADA@example.com").await;

    let response = sign_in(&app, &google).await;

    assert_sign_in_failed(&response, "email_already_registered");
    assert_eq!(count(&db, "users").await, 1);
    assert_eq!(count(&db, "oauth_accounts").await, 0);
}

// ---------------------------------------------------------------------------------------------
// Linking
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn link_requires_an_access_token(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db);

    let response = TestRequest::post("/api/v1/auth/google/link")
        .send(&app)
        .await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
    assert_eq!(response.body["error"]["code"], "unauthorized");
}

#[sqlx::test]
async fn link_attaches_the_google_account_to_the_signed_in_user(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db.clone());
    let access_token = register_and_login(&app, "ada.work@example.com").await;
    let flow = start_link(&app, &access_token).await;
    google.respond_with(claims(&flow.nonce));

    let response = callback(&app, &flow).await;

    assert_eq!(response.status, StatusCode::SEE_OTHER);
    assert_eq!(
        response.location(),
        format!("{FRONTEND_ORIGIN}/settings?google=linked")
    );
    assert!(response.set_cookie(REFRESH_COOKIE).is_none());
    assert_eq!(count(&db, "users").await, 1);
    assert_eq!(count(&db, "oauth_accounts").await, 1);
}

#[sqlx::test]
async fn linked_google_account_signs_in_as_the_linked_user(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db);
    let access_token = register_and_login(&app, "ada.work@example.com").await;
    let flow = start_link(&app, &access_token).await;
    google.respond_with(claims(&flow.nonce));
    callback(&app, &flow).await;

    let signed_in = sign_in(&app, &google).await;

    let session = TestRequest::post("/api/v1/auth/refresh")
        .cookie(
            REFRESH_COOKIE,
            signed_in.cookie_value(REFRESH_COOKIE).unwrap(),
        )
        .send(&app)
        .await;
    assert_eq!(session.body["user"]["email"], "ada.work@example.com");
}

#[sqlx::test]
async fn link_is_idempotent_for_the_same_user_and_google_account(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db.clone());
    let access_token = register_and_login(&app, "ada.work@example.com").await;
    for _ in 0..2 {
        let flow = start_link(&app, &access_token).await;
        google.respond_with(claims(&flow.nonce));
        let response = callback(&app, &flow).await;
        assert_eq!(
            response.location(),
            format!("{FRONTEND_ORIGIN}/settings?google=linked")
        );
    }

    assert_eq!(count(&db, "oauth_accounts").await, 1);
}

#[sqlx::test]
async fn link_rejects_a_google_account_that_belongs_to_another_user(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db.clone());
    sign_in(&app, &google).await;
    let other_user = register_and_login(&app, "grace@example.com").await;
    let flow = start_link(&app, &other_user).await;
    google.respond_with(claims(&flow.nonce));

    let response = callback(&app, &flow).await;

    assert_eq!(response.status, StatusCode::SEE_OTHER);
    assert_eq!(
        response.location(),
        format!("{FRONTEND_ORIGIN}/settings?error=google_account_already_linked")
    );
    assert_eq!(count(&db, "oauth_accounts").await, 1);
}

#[sqlx::test]
async fn link_rejects_a_second_google_account_for_the_same_user(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db.clone());
    let access_token = register_and_login(&app, "ada.work@example.com").await;
    let first = start_link(&app, &access_token).await;
    google.respond_with(claims(&first.nonce));
    callback(&app, &first).await;
    let second = start_link(&app, &access_token).await;
    let mut other_account = claims(&second.nonce);
    other_account["sub"] = json!("google-subject-2");
    google.respond_with(other_account);

    let response = callback(&app, &second).await;

    assert_eq!(
        response.location(),
        format!("{FRONTEND_ORIGIN}/settings?error=google_account_already_linked")
    );
    assert_eq!(count(&db, "oauth_accounts").await, 1);
}

#[sqlx::test]
async fn link_failure_from_a_bad_id_token_redirects_to_settings(db: PgPool) {
    let google = MockGoogle::start().await;
    let app = google.app(db.clone());
    let access_token = register_and_login(&app, "ada.work@example.com").await;
    let flow = start_link(&app, &access_token).await;
    google.respond_with(claims("nonce-from-another-flow"));

    let response = callback(&app, &flow).await;

    assert_eq!(
        response.location(),
        format!("{FRONTEND_ORIGIN}/settings?error=google_sign_in_failed")
    );
    assert_eq!(count(&db, "oauth_accounts").await, 0);
}
