//! Integration tests for the account fields of `GET /me` and `PATCH /me`.

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

async fn get_me(app: &Router, token: &str) -> TestResponse {
    TestRequest::get("/api/v1/me").bearer(token).send(app).await
}

async fn patch_me(app: &Router, token: &str, body: Value) -> TestResponse {
    TestRequest::new(Method::PATCH, "/api/v1/me")
        .bearer(token)
        .json(&body)
        .send(app)
        .await
}

fn field_codes<'a>(response: &'a TestResponse, field: &str) -> Vec<&'a str> {
    response.body["error"]["details"]["fields"][field]
        .as_array()
        .unwrap()
        .iter()
        .map(|code| code.as_str().unwrap())
        .collect()
}

#[sqlx::test]
async fn me_returns_the_default_currency(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = get_me(&app, &token).await;

    assert_eq!(response.body["currency"], "VND");
}

#[sqlx::test]
async fn patch_updates_every_account_field(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = patch_me(
        &app,
        &token,
        json!({
            "display_name": "  Ada L.  ",
            "timezone": "Europe/London",
            "locale": "vi",
            "currency": "USD",
        }),
    )
    .await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["display_name"], "Ada L.");
    assert_eq!(response.body["timezone"], "Europe/London");
    assert_eq!(response.body["locale"], "vi");
    assert_eq!(response.body["currency"], "USD");
    assert_eq!(response.body["email"], ADA);
    assert_eq!(get_me(&app, &token).await.body, response.body);
}

#[sqlx::test]
async fn patch_changes_only_the_fields_present(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let before = get_me(&app, &token).await.body;

    let response = patch_me(&app, &token, json!({ "currency": "EUR" })).await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["currency"], "EUR");
    assert_eq!(response.body["display_name"], before["display_name"]);
    assert_eq!(response.body["timezone"], before["timezone"]);
    assert_eq!(response.body["locale"], before["locale"]);
}

#[sqlx::test]
async fn patch_rejects_unsupported_values(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let cases = [
        ("currency", json!("XXX"), "currency"),
        ("currency", json!("usd"), "currency"),
        ("currency", json!(""), "currency"),
        ("timezone", json!("Mars/Olympus"), "timezone"),
        ("timezone", json!(""), "timezone"),
        ("display_name", json!("   "), "blank"),
        ("display_name", json!("x".repeat(101)), "length"),
    ];
    for (field, value, code) in cases {
        let response = patch_me(&app, &token, json!({ field: value })).await;

        assert_eq!(
            response.status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{field} = {value}"
        );
        assert_eq!(field_codes(&response, field), [code], "{field} = {value}");
    }
}

#[sqlx::test]
async fn patch_rejects_an_unknown_language(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = patch_me(&app, &token, json!({ "locale": "fr" })).await;

    assert_eq!(response.status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn every_supported_currency_is_accepted(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    for currency in [
        "VND", "USD", "EUR", "GBP", "JPY", "KRW", "CNY", "SGD", "THB", "AUD",
    ] {
        let response = patch_me(&app, &token, json!({ "currency": currency })).await;

        assert_eq!(response.status, StatusCode::OK, "{currency}");
        assert_eq!(response.body["currency"], currency);
    }
}

#[sqlx::test]
async fn patch_does_not_touch_another_user(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;

    patch_me(&app, &ada, json!({ "currency": "USD" })).await;

    assert_eq!(get_me(&app, &grace).await.body["currency"], "VND");
}

#[sqlx::test]
async fn patch_requires_authentication(db: PgPool) {
    let app = common::app(db);

    let response = TestRequest::new(Method::PATCH, "/api/v1/me")
        .json(&json!({ "currency": "USD" }))
        .send(&app)
        .await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
}
