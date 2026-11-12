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
async fn put_replaces_the_settings_and_get_returns_them(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = put_settings(&app, &token, valid_body()).await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body, valid_body());
    assert_eq!(get_settings(&app, &token).await.body, valid_body());
}

#[sqlx::test]
async fn put_accepts_the_boundary_values(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    for body in [
        json!({ "focus_minutes": 1, "short_break_minutes": 1, "long_break_minutes": 1, "long_break_every": 1 }),
        json!({ "focus_minutes": 180, "short_break_minutes": 60, "long_break_minutes": 120, "long_break_every": 12 }),
    ] {
        let response = put_settings(&app, &token, body.clone()).await;
        assert_eq!(response.status, StatusCode::OK, "{body}");
    }
}

#[sqlx::test]
async fn put_rejects_values_outside_the_allowed_range(db: PgPool) {
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
        assert_eq!(
            response.body["error"]["details"]["fields"][field],
            json!(["range"]),
            "{field}={value}"
        );
    }

    // Nothing was stored by the rejected requests.
    assert_eq!(get_settings(&app, &token).await.body["focus_minutes"], 25);
}

#[sqlx::test]
async fn put_rejects_a_missing_or_malformed_field(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let missing = put_settings(&app, &token, json!({ "focus_minutes": 25 })).await;
    assert_eq!(missing.status, StatusCode::BAD_REQUEST);
    assert_eq!(missing.body["error"]["code"], "invalid_request");

    let mut body = valid_body();
    body["focus_minutes"] = json!("25");
    let wrong_type = put_settings(&app, &token, body).await;
    assert_eq!(wrong_type.status, StatusCode::BAD_REQUEST);

    let mut body = valid_body();
    body["focus_minutes"] = json!(100_000);
    let overflow = put_settings(&app, &token, body).await;
    assert_eq!(overflow.status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn settings_of_one_user_do_not_affect_another(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;

    put_settings(&app, &ada, valid_body()).await;

    assert_eq!(get_settings(&app, &grace).await.body["focus_minutes"], 25);
    assert_eq!(get_settings(&app, &ada).await.body, valid_body());
}

#[sqlx::test]
async fn settings_require_authentication(db: PgPool) {
    let app = common::app(db);

    let get = TestRequest::get("/api/v1/settings").send(&app).await;
    assert_eq!(get.status, StatusCode::UNAUTHORIZED);

    let put = TestRequest::new(Method::PUT, "/api/v1/settings")
        .json(&valid_body())
        .send(&app)
        .await;
    assert_eq!(put.status, StatusCode::UNAUTHORIZED);
}
