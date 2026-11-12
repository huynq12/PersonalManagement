//! Helpers shared by the integration tests.

// Each test binary compiles this module separately and uses a different subset of it.
#![allow(dead_code, clippy::unwrap_used, clippy::missing_panics_doc)]

use axum::{
    Router,
    body::Body,
    http::{HeaderMap, Method, Request, StatusCode, header},
};
use http_body_util::BodyExt;
use personal_management_api::{
    AppState,
    config::{Config, Secret},
};
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;

/// Origin of the frontend used in tests.
pub const FRONTEND_ORIGIN: &str = "http://localhost:5173";
/// Signing key used in tests.
pub const JWT_SECRET: &str = "test-secret-test-secret-test-secret-test";

/// Configuration for tests: no Google client, cookies without `Secure`.
pub fn config() -> Config {
    Config {
        database_url: Secret::new("unused: tests receive a pool from sqlx::test"),
        port: 0,
        cors_origin: FRONTEND_ORIGIN.to_owned(),
        jwt_secret: Secret::new(JWT_SECRET),
        cookie_secure: false,
        google: None,
    }
}

/// Builds the application with the default test configuration.
pub fn app(db: PgPool) -> Router {
    app_with(db, config())
}

/// Builds the application with a custom configuration.
pub fn app_with(db: PgPool, config: Config) -> Router {
    personal_management_api::app(AppState::new(db, config).unwrap()).unwrap()
}

/// A response with its body already read and parsed.
#[derive(Debug)]
pub struct TestResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    /// The JSON body, or `Value::Null` when the body is empty.
    pub body: Value,
}

impl TestResponse {
    /// The raw `Set-Cookie` header for the cookie called `name`, if the response sets it.
    pub fn set_cookie(&self, name: &str) -> Option<&str> {
        let prefix = format!("{name}=");
        self.headers
            .get_all(header::SET_COOKIE)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .find(|value| value.starts_with(&prefix))
    }

    /// The value of the cookie called `name`, if the response sets it.
    pub fn cookie_value(&self, name: &str) -> Option<&str> {
        let raw = self.set_cookie(name)?;
        let pair = raw.split(';').next()?;
        pair.split_once('=').map(|(_, value)| value)
    }

    /// The `Location` header of a redirect.
    pub fn location(&self) -> &str {
        self.headers[header::LOCATION].to_str().unwrap()
    }
}

/// Sends a request to the application and reads the whole response.
pub async fn send(app: &Router, request: Request<Body>) -> TestResponse {
    let response = app.clone().oneshot(request).await.unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };

    TestResponse {
        status,
        headers,
        body,
    }
}

/// Builder for test requests.
pub struct TestRequest {
    builder: axum::http::request::Builder,
    body: Body,
}

impl TestRequest {
    pub fn new(method: Method, uri: &str) -> Self {
        Self {
            builder: Request::builder().method(method).uri(uri),
            body: Body::empty(),
        }
    }

    pub fn get(uri: &str) -> Self {
        Self::new(Method::GET, uri)
    }

    pub fn post(uri: &str) -> Self {
        Self::new(Method::POST, uri)
    }

    /// Sets a JSON body.
    pub fn json(mut self, body: &Value) -> Self {
        self.builder = self
            .builder
            .header(header::CONTENT_TYPE, "application/json");
        self.body = Body::from(body.to_string());
        self
    }

    /// Sets a raw body declared as JSON, for malformed input.
    pub fn raw_json(mut self, body: &'static str) -> Self {
        self.builder = self
            .builder
            .header(header::CONTENT_TYPE, "application/json");
        self.body = Body::from(body);
        self
    }

    /// Sends an access token.
    pub fn bearer(mut self, token: &str) -> Self {
        self.builder = self
            .builder
            .header(header::AUTHORIZATION, format!("Bearer {token}"));
        self
    }

    /// Sends one cookie.
    pub fn cookie(mut self, name: &str, value: &str) -> Self {
        self.builder = self
            .builder
            .header(header::COOKIE, format!("{name}={value}"));
        self
    }

    pub async fn send(self, app: &Router) -> TestResponse {
        send(app, self.builder.body(self.body).unwrap()).await
    }
}

/// Password used for every account created by [`sign_up`].
pub const PASSWORD: &str = "correct horse battery";

/// Registers an account, signs in, and returns the access token.
pub async fn sign_up(app: &Router, email: &str) -> String {
    let registered = TestRequest::post("/api/v1/auth/register")
        .json(&serde_json::json!({ "email": email, "password": PASSWORD }))
        .send(app)
        .await;
    assert_eq!(registered.status, StatusCode::CREATED);

    let session = TestRequest::post("/api/v1/auth/login")
        .json(&serde_json::json!({ "email": email, "password": PASSWORD }))
        .send(app)
        .await;
    assert_eq!(session.status, StatusCode::OK);
    session.body["access_token"].as_str().unwrap().to_owned()
}

/// The current calendar date in the default user time zone, as the database sees it.
pub async fn today(db: &PgPool) -> chrono::NaiveDate {
    sqlx::query_scalar("SELECT (now() AT TIME ZONE 'Asia/Ho_Chi_Minh')::date")
        .fetch_one(db)
        .await
        .unwrap()
}
