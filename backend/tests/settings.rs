//! Integration tests for the Pomodoro settings endpoints.

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

async fn get_settings(app: &Router, token: &str) -> TestResponse {
    TestRequest::get("/api/v1/settings")
        .bearer(token)
        .send(app)
        .await
}

async fn put_settings(app: &Router, token: &str, body: Value) -> TestResponse {
    TestRequest::new(Method::PUT, "/api/v1/settings")
        .bearer(token)
        .json(&body)
        .send(app)
        .await
}

fn valid_body() -> Value {
    json!({
        "focus_minutes": 50,
        "short_break_minutes": 10,
        "long_break_minutes": 30,
        "long_break_every": 3,
    })
}

#[sqlx::test]
async fn get_returns_the_defaults_for_a_new_user(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = get_settings(&app, &token).await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(
        response.body,
        json!({
            "focus_minutes": 25,
            "short_break_minutes": 5,
            "long_break_minutes": 15,
            "long_break_every": 4,
        })
    );
}

#[sqlx::test]
async fn put_replaces_the_settings(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = put_settings(&app, &token, valid_body()).await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body, valid_body());
    assert_eq!(get_settings(&app, &token).await.body, valid_body());
}

#[sqlx::test]
async fn put_accepts_the_limits(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    for body in [
        json!({ "focus_minutes": 1, "short_break_minutes": 1, "long_break_minutes": 1, "long_break_every": 1 }),
        json!({ "focus_minutes": 180, "short_break_minutes": 60, "long_break_minutes": 120, "long_break_every": 12 }),
    ] {
        let response = put_settings(&app, &token, body.clone()).await;

        assert_eq!(response.status, StatusCode::OK, "{body}");
        assert_eq!(response.body, body);
    }
}

#[sqlx::test]
async fn put_rejects_values_out_of_range(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let cases = [
        ("focus_minutes", 0),
        ("focus_minutes", 181),
        ("short_break_minutes", 0),
        ("short_break_minutes", 61),
        ("long_break_minutes", 0),
        ("long_break_minutes", 121),
        ("long_break_every", 0),
        ("long_break_every", 13),
    ];

    for (field, value) in cases {
        let mut body = valid_body();
        body[field] = json!(value);

        let response = put_settings(&app, &token, body).await;

        assert_eq!(
            response.status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{field}={value}"
        );
        assert_eq!(response.body["error"]["code"], "validation_failed");
        assert!(
            response.body["error"]["details"]["fields"][field].is_array(),
            "{field}={value}"
        );
    }
    // Nothing was saved by the rejected requests.
    assert_eq!(get_settings(&app, &token).await.body["focus_minutes"], 25);
}

#[sqlx::test]
async fn put_requires_every_field(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = put_settings(&app, &token, json!({ "focus_minutes": 30 })).await;

    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(response.body["error"]["code"], "invalid_request");
}

#[sqlx::test]
async fn settings_belong_to_one_user(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;

    put_settings(&app, &ada, valid_body()).await;

    assert_eq!(get_settings(&app, &grace).await.body["focus_minutes"], 25);
    assert_eq!(get_settings(&app, &ada).await.body["focus_minutes"], 50);
}

#[sqlx::test]
async fn settings_require_authentication(db: PgPool) {
    let app = common::app(db);

    let read = TestRequest::get("/api/v1/settings").send(&app).await;
    let write = TestRequest::new(Method::PUT, "/api/v1/settings")
        .json(&valid_body())
        .send(&app)
        .await;

    assert_eq!(read.status, StatusCode::UNAUTHORIZED);
    assert_eq!(write.status, StatusCode::UNAUTHORIZED);
}
