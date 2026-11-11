//! Integration tests for email + password auth: register, login, refresh, logout, and `/me`.

// Test helpers are not `#[test]` functions, so the clippy.toml test exemption does not reach them.
#![allow(clippy::unwrap_used)]

use axum::{Router, http::StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::{TestRequest, TestResponse};

mod common;

const EMAIL: &str = "ada@example.com";
const PASSWORD: &str = "correct horse battery";
const REFRESH_COOKIE: &str = "refresh_token";

async fn register(app: &Router, email: &str, password: &str) -> TestResponse {
    TestRequest::post("/api/v1/auth/register")
        .json(&json!({ "email": email, "password": password, "display_name": "Ada" }))
        .send(app)
        .await
}

async fn login(app: &Router, email: &str, password: &str) -> TestResponse {
    TestRequest::post("/api/v1/auth/login")
        .json(&json!({ "email": email, "password": password }))
        .send(app)
        .await
}

async fn refresh(app: &Router, refresh_token: &str) -> TestResponse {
    TestRequest::post("/api/v1/auth/refresh")
        .cookie(REFRESH_COOKIE, refresh_token)
        .send(app)
        .await
}

/// Registers the default user, signs in, and returns the login response.
async fn register_and_login(app: &Router) -> TestResponse {
    assert_eq!(
        register(app, EMAIL, PASSWORD).await.status,
        StatusCode::CREATED
    );
    let response = login(app, EMAIL, PASSWORD).await;
    assert_eq!(response.status, StatusCode::OK);
    response
}

fn refresh_token(response: &TestResponse) -> String {
    response.cookie_value(REFRESH_COOKIE).unwrap().to_owned()
}

fn access_token(response: &TestResponse) -> String {
    response.body["access_token"].as_str().unwrap().to_owned()
}

fn error_code(response: &TestResponse) -> &Value {
    &response.body["error"]["code"]
}

// ---------------------------------------------------------------------------------------------
// Register
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn register_creates_user_without_signing_in(db: PgPool) {
    let app = common::app(db);

    let response = register(&app, EMAIL, PASSWORD).await;

    assert_eq!(response.status, StatusCode::CREATED);
    assert_eq!(response.body["email"], EMAIL);
    assert_eq!(response.body["display_name"], "Ada");
    assert_eq!(response.body["locale"], "en");
    assert_eq!(response.body["timezone"], "Asia/Ho_Chi_Minh");
    assert!(response.body["id"].is_string());
    assert!(response.body.get("access_token").is_none());
    assert!(response.set_cookie(REFRESH_COOKIE).is_none());
}

#[sqlx::test]
async fn register_never_returns_or_stores_the_plain_password(db: PgPool) {
    let app = common::app(db.clone());

    let response = register(&app, EMAIL, PASSWORD).await;

    assert!(!response.body.to_string().contains(PASSWORD));
    assert!(response.body.get("password_hash").is_none());
    let stored: String = sqlx::query_scalar("SELECT password_hash FROM users")
        .fetch_one(&db)
        .await
        .unwrap();
    assert!(stored.starts_with("$argon2id$"));
}

#[sqlx::test]
async fn register_creates_default_pomodoro_settings(db: PgPool) {
    let app = common::app(db.clone());

    register(&app, EMAIL, PASSWORD).await;

    let focus_minutes: i16 = sqlx::query_scalar("SELECT focus_minutes FROM user_settings")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(focus_minutes, 25);
}

#[sqlx::test]
async fn register_uses_the_email_local_part_when_display_name_is_missing(db: PgPool) {
    let app = common::app(db);

    let response = TestRequest::post("/api/v1/auth/register")
        .json(&json!({ "email": EMAIL, "password": PASSWORD, "locale": "vi" }))
        .send(&app)
        .await;

    assert_eq!(response.status, StatusCode::CREATED);
    assert_eq!(response.body["display_name"], "ada");
    assert_eq!(response.body["locale"], "vi");
}

#[sqlx::test]
async fn register_rejects_invalid_email(db: PgPool) {
    let app = common::app(db);

    let response = register(&app, "not-an-email", PASSWORD).await;

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(*error_code(&response), "validation_failed");
    assert_eq!(
        response.body["error"]["details"]["fields"]["email"],
        json!(["email"])
    );
}

#[sqlx::test]
async fn register_rejects_password_shorter_than_8_characters(db: PgPool) {
    let app = common::app(db);

    let response = register(&app, EMAIL, "1234567").await;

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        response.body["error"]["details"]["fields"]["password"],
        json!(["length"])
    );
}

#[sqlx::test]
async fn register_rejects_password_longer_than_128_characters(db: PgPool) {
    let app = common::app(db);

    let response = register(&app, EMAIL, &"a".repeat(129)).await;

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(*error_code(&response), "validation_failed");
}

#[sqlx::test]
async fn register_rejects_unsupported_locale(db: PgPool) {
    let app = common::app(db);

    let response = TestRequest::post("/api/v1/auth/register")
        .json(&json!({ "email": EMAIL, "password": PASSWORD, "locale": "fr" }))
        .send(&app)
        .await;

    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(*error_code(&response), "invalid_request");
}

#[sqlx::test]
async fn register_rejects_malformed_json(db: PgPool) {
    let app = common::app(db);

    let response = TestRequest::post("/api/v1/auth/register")
        .raw_json("{ not json")
        .send(&app)
        .await;

    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(*error_code(&response), "invalid_request");
}

#[sqlx::test]
async fn register_rejects_email_already_taken_ignoring_case(db: PgPool) {
    let app = common::app(db);
    register(&app, EMAIL, PASSWORD).await;

    let response = register(&app, "ADA@Example.com", PASSWORD).await;

    assert_eq!(response.status, StatusCode::CONFLICT);
    assert_eq!(*error_code(&response), "email_taken");
}

// ---------------------------------------------------------------------------------------------
// Login
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn login_returns_access_token_and_user(db: PgPool) {
    let app = common::app(db);

    let response = register_and_login(&app).await;

    assert!(!access_token(&response).is_empty());
    assert_eq!(response.body["token_type"], "Bearer");
    assert_eq!(response.body["expires_in"], 900);
    assert_eq!(response.body["user"]["email"], EMAIL);
}

#[sqlx::test]
async fn login_sets_http_only_refresh_cookie_scoped_to_auth_routes(db: PgPool) {
    let app = common::app(db);

    let response = register_and_login(&app).await;

    let cookie = response.set_cookie(REFRESH_COOKIE).unwrap();
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("Path=/api/v1/auth"));
    assert!(cookie.contains("SameSite=Lax"));
    assert!(cookie.contains("Max-Age=2592000"));
    assert!(!cookie.contains("Secure"));
}

#[sqlx::test]
async fn login_marks_cookie_secure_and_cross_site_when_configured(db: PgPool) {
    let mut config = common::config();
    config.cookie_secure = true;
    let app = common::app_with(db, config);

    let response = register_and_login(&app).await;

    let cookie = response.set_cookie(REFRESH_COOKIE).unwrap();
    assert!(cookie.contains("Secure"));
    assert!(cookie.contains("SameSite=None"));
}

#[sqlx::test]
async fn login_stores_only_a_hash_of_the_refresh_token(db: PgPool) {
    let app = common::app(db.clone());

    let response = register_and_login(&app).await;

    let stored: String = sqlx::query_scalar("SELECT token_hash FROM refresh_tokens")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_ne!(stored, refresh_token(&response));
}

#[sqlx::test]
async fn login_accepts_email_in_a_different_case(db: PgPool) {
    let app = common::app(db);
    register(&app, EMAIL, PASSWORD).await;

    let response = login(&app, "ADA@EXAMPLE.COM", PASSWORD).await;

    assert_eq!(response.status, StatusCode::OK);
}

#[sqlx::test]
async fn login_rejects_wrong_password(db: PgPool) {
    let app = common::app(db);
    register(&app, EMAIL, PASSWORD).await;

    let response = login(&app, EMAIL, "wrong password").await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
    assert_eq!(*error_code(&response), "invalid_credentials");
    assert!(response.set_cookie(REFRESH_COOKIE).is_none());
}

#[sqlx::test]
async fn login_rejects_unknown_email_with_the_same_error(db: PgPool) {
    let app = common::app(db);

    let response = login(&app, "nobody@example.com", PASSWORD).await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
    assert_eq!(*error_code(&response), "invalid_credentials");
}

#[sqlx::test]
async fn login_rejects_missing_fields(db: PgPool) {
    let app = common::app(db);

    let response = TestRequest::post("/api/v1/auth/login")
        .json(&json!({ "email": EMAIL }))
        .send(&app)
        .await;

    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(*error_code(&response), "invalid_request");
}

// ---------------------------------------------------------------------------------------------
// Current user
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn me_returns_the_user_who_owns_the_token(db: PgPool) {
    let app = common::app(db);
    let ada = register_and_login(&app).await;
    register(&app, "grace@example.com", PASSWORD).await;
    let grace = login(&app, "grace@example.com", PASSWORD).await;

    let ada_me = TestRequest::get("/api/v1/me")
        .bearer(&access_token(&ada))
        .send(&app)
        .await;
    let grace_me = TestRequest::get("/api/v1/me")
        .bearer(&access_token(&grace))
        .send(&app)
        .await;

    assert_eq!(ada_me.status, StatusCode::OK);
    assert_eq!(ada_me.body["email"], EMAIL);
    assert_eq!(grace_me.body["email"], "grace@example.com");
}

#[sqlx::test]
async fn me_rejects_request_without_token(db: PgPool) {
    let app = common::app(db);

    let response = TestRequest::get("/api/v1/me").send(&app).await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
    assert_eq!(*error_code(&response), "unauthorized");
}

#[sqlx::test]
async fn me_rejects_malformed_token(db: PgPool) {
    let app = common::app(db);

    let response = TestRequest::get("/api/v1/me")
        .bearer("not.a.jwt")
        .send(&app)
        .await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
    assert_eq!(*error_code(&response), "unauthorized");
}

#[sqlx::test]
async fn me_rejects_token_signed_with_another_secret(db: PgPool) {
    let app = common::app(db.clone());
    let mut other_config = common::config();
    other_config.jwt_secret =
        personal_management_api::config::Secret::new("another-secret-another-secret-another");
    let other_app = common::app_with(db, other_config);
    let session = register_and_login(&other_app).await;

    let response = TestRequest::get("/api/v1/me")
        .bearer(&access_token(&session))
        .send(&app)
        .await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn me_rejects_a_refresh_token_used_as_access_token(db: PgPool) {
    let app = common::app(db);
    let session = register_and_login(&app).await;

    let response = TestRequest::get("/api/v1/me")
        .bearer(&refresh_token(&session))
        .send(&app)
        .await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn me_rejects_token_of_a_deleted_user(db: PgPool) {
    let app = common::app(db.clone());
    let session = register_and_login(&app).await;
    sqlx::query("DELETE FROM users").execute(&db).await.unwrap();

    let response = TestRequest::get("/api/v1/me")
        .bearer(&access_token(&session))
        .send(&app)
        .await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------------------------
// Refresh
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn refresh_returns_new_access_token_and_rotates_the_cookie(db: PgPool) {
    let app = common::app(db);
    let session = register_and_login(&app).await;
    let old_token = refresh_token(&session);

    let response = refresh(&app, &old_token).await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["user"]["email"], EMAIL);
    let new_token = refresh_token(&response);
    assert_ne!(new_token, old_token);
    let me = TestRequest::get("/api/v1/me")
        .bearer(&access_token(&response))
        .send(&app)
        .await;
    assert_eq!(me.status, StatusCode::OK);
}

#[sqlx::test]
async fn refresh_accepts_the_rotated_token(db: PgPool) {
    let app = common::app(db);
    let session = register_and_login(&app).await;
    let rotated = refresh(&app, &refresh_token(&session)).await;

    let response = refresh(&app, &refresh_token(&rotated)).await;

    assert_eq!(response.status, StatusCode::OK);
}

#[sqlx::test]
async fn refresh_rejects_request_without_cookie(db: PgPool) {
    let app = common::app(db);

    let response = TestRequest::post("/api/v1/auth/refresh").send(&app).await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
    assert_eq!(*error_code(&response), "invalid_refresh_token");
}

#[sqlx::test]
async fn refresh_rejects_unknown_token(db: PgPool) {
    let app = common::app(db);

    let response = refresh(&app, "made-up-token").await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
    assert_eq!(*error_code(&response), "invalid_refresh_token");
}

#[sqlx::test]
async fn refresh_rejects_expired_token(db: PgPool) {
    let app = common::app(db.clone());
    let session = register_and_login(&app).await;
    sqlx::query("UPDATE refresh_tokens SET expires_at = now() - interval '1 second'")
        .execute(&db)
        .await
        .unwrap();

    let response = refresh(&app, &refresh_token(&session)).await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
    assert_eq!(*error_code(&response), "invalid_refresh_token");
}

#[sqlx::test]
async fn refresh_reuse_of_a_rotated_token_revokes_its_whole_family(db: PgPool) {
    let app = common::app(db);
    let session = register_and_login(&app).await;
    let stolen = refresh_token(&session);
    let rotated = refresh(&app, &stolen).await;

    let reuse = refresh(&app, &stolen).await;
    let after_reuse = refresh(&app, &refresh_token(&rotated)).await;

    assert_eq!(reuse.status, StatusCode::UNAUTHORIZED);
    assert_eq!(*error_code(&reuse), "invalid_refresh_token");
    assert_eq!(after_reuse.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn refresh_reuse_does_not_revoke_other_sign_ins(db: PgPool) {
    let app = common::app(db);
    let first_device = register_and_login(&app).await;
    let second_device = login(&app, EMAIL, PASSWORD).await;
    let stolen = refresh_token(&first_device);
    refresh(&app, &stolen).await;
    refresh(&app, &stolen).await;

    let response = refresh(&app, &refresh_token(&second_device)).await;

    assert_eq!(response.status, StatusCode::OK);
}

// ---------------------------------------------------------------------------------------------
// Logout
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn logout_revokes_the_refresh_token_and_clears_the_cookie(db: PgPool) {
    let app = common::app(db);
    let session = register_and_login(&app).await;
    let token = refresh_token(&session);

    let response = TestRequest::post("/api/v1/auth/logout")
        .cookie(REFRESH_COOKIE, &token)
        .send(&app)
        .await;

    assert_eq!(response.status, StatusCode::NO_CONTENT);
    let cleared = response.set_cookie(REFRESH_COOKIE).unwrap();
    assert!(cleared.contains("Max-Age=0"));
    assert_eq!(refresh(&app, &token).await.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn logout_without_cookie_still_succeeds(db: PgPool) {
    let app = common::app(db);

    let response = TestRequest::post("/api/v1/auth/logout").send(&app).await;

    assert_eq!(response.status, StatusCode::NO_CONTENT);
}
