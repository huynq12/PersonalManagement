//! Integration tests for `PATCH /me`.

// Test helpers are not `#[test]` functions, so the clippy.toml test exemption does not reach them.
#![allow(clippy::unwrap_used)]

use axum::{
    Router,
    http::{Method, StatusCode},
};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::{TestRequest, TestResponse, sign_up};

mod common;

const ADA: &str = "ada@example.com";
const GRACE: &str = "grace@example.com";

async fn patch_me(app: &Router, token: &str, body: Value) -> TestResponse {
    TestRequest::new(Method::PATCH, "/api/v1/me")
        .bearer(token)
        .json(&body)
        .send(app)
        .await
}

async fn get_me(app: &Router, token: &str) -> TestResponse {
    TestRequest::get("/api/v1/me").bearer(token).send(app).await
}

fn field_errors<'a>(response: &'a TestResponse, field: &str) -> &'a Value {
    &response.body["error"]["details"]["fields"][field]
}

#[sqlx::test]
async fn patch_updates_every_given_field(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = patch_me(
        &app,
        &token,
        json!({ "display_name": "Ada L.", "timezone": "Europe/London", "locale": "en" }),
    )
    .await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["display_name"], "Ada L.");
    assert_eq!(response.body["timezone"], "Europe/London");
    assert_eq!(response.body["locale"], "en");
    assert_eq!(response.body["email"], ADA);
    assert_eq!(get_me(&app, &token).await.body, response.body);
}

#[sqlx::test]
async fn patch_keeps_the_fields_that_are_left_out(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let before = get_me(&app, &token).await.body;

    let response = patch_me(&app, &token, json!({ "locale": "vi" })).await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["locale"], "vi");
    assert_eq!(response.body["display_name"], before["display_name"]);
    assert_eq!(response.body["timezone"], before["timezone"]);

    let empty = patch_me(&app, &token, json!({})).await;
    assert_eq!(empty.status, StatusCode::OK);
    assert_eq!(empty.body["locale"], "vi");
}

#[sqlx::test]
async fn patch_trims_the_display_name(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = patch_me(&app, &token, json!({ "display_name": "  Ada  " })).await;

    assert_eq!(response.body["display_name"], "Ada");
}

#[sqlx::test]
async fn patch_rejects_a_blank_or_too_long_display_name(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let blank = patch_me(&app, &token, json!({ "display_name": "   " })).await;
    assert_eq!(blank.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(field_errors(&blank, "display_name"), &json!(["blank"]));

    let long = patch_me(&app, &token, json!({ "display_name": "a".repeat(101) })).await;
    assert_eq!(long.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(field_errors(&long, "display_name"), &json!(["length"]));
}

#[sqlx::test]
async fn patch_rejects_an_unknown_time_zone(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    for timezone in ["", "Mars/Olympus", "UTC+7", "asia/ho_chi_minh "] {
        let response = patch_me(&app, &token, json!({ "timezone": timezone })).await;

        assert_eq!(
            response.status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{timezone:?}"
        );
        assert_eq!(field_errors(&response, "timezone"), &json!(["timezone"]));
    }
    assert_eq!(
        get_me(&app, &token).await.body["timezone"],
        "Asia/Ho_Chi_Minh"
    );
}

#[sqlx::test]
async fn patch_rejects_an_unsupported_locale(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = patch_me(&app, &token, json!({ "locale": "fr" })).await;

    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(response.body["error"]["code"], "invalid_request");
}

#[sqlx::test]
async fn patch_ignores_attempts_to_change_the_email(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = patch_me(&app, &token, json!({ "email": GRACE })).await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["email"], ADA);
}

#[sqlx::test]
async fn patch_only_changes_the_signed_in_user(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;

    patch_me(&app, &ada, json!({ "display_name": "Ada L." })).await;

    assert_ne!(get_me(&app, &grace).await.body["display_name"], "Ada L.");
}

#[sqlx::test]
async fn patch_requires_authentication(db: PgPool) {
    let app = common::app(db);

    let response = TestRequest::new(Method::PATCH, "/api/v1/me")
        .json(&json!({ "locale": "vi" }))
        .send(&app)
        .await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
}
