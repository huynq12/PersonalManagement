//! Integration tests for `GET /settings` and `PATCH /settings`.

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

async fn patch_settings(app: &Router, token: &str, body: Value) -> TestResponse {
    TestRequest::new(Method::PATCH, "/api/v1/settings")
        .bearer(token)
        .json(&body)
        .send(app)
        .await
}

fn error_code(response: &TestResponse) -> &str {
    response.body["error"]["code"].as_str().unwrap()
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
            "hourly_wage": null,
            "base_work_hours": null,
            "monthly_budget": null,
        })
    );
}

#[sqlx::test]
async fn get_works_for_a_user_who_has_no_settings_row(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    sqlx::query("DELETE FROM user_settings")
        .execute(&db)
        .await
        .unwrap();

    let response = get_settings(&app, &token).await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["focus_minutes"], 25);
    assert_eq!(response.body["hourly_wage"], Value::Null);
}

#[sqlx::test]
async fn patch_works_for_a_user_who_has_no_settings_row(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    sqlx::query("DELETE FROM user_settings")
        .execute(&db)
        .await
        .unwrap();

    let response = patch_settings(&app, &token, json!({ "hourly_wage": 30000 })).await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["hourly_wage"], 30000);
    assert_eq!(response.body["focus_minutes"], 25);
}

#[sqlx::test]
async fn patch_updates_only_the_fields_present(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    patch_settings(
        &app,
        &token,
        json!({ "hourly_wage": 30000, "monthly_budget": 5_000_000 }),
    )
    .await;

    let response = patch_settings(&app, &token, json!({ "focus_minutes": 50 })).await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["focus_minutes"], 50);
    assert_eq!(response.body["short_break_minutes"], 5);
    assert_eq!(response.body["hourly_wage"], 30000);
    assert_eq!(response.body["monthly_budget"], 5_000_000);
    assert_eq!(get_settings(&app, &token).await.body, response.body);
}

#[sqlx::test]
async fn patch_with_an_empty_body_changes_nothing(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = patch_settings(&app, &token, json!({})).await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body, get_settings(&app, &token).await.body);
}

#[sqlx::test]
async fn patch_stores_base_hours_with_two_decimals(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = patch_settings(&app, &token, json!({ "base_work_hours": 7.25 })).await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["base_work_hours"], 7.25);
    assert_eq!(
        get_settings(&app, &token).await.body["base_work_hours"],
        7.25
    );
}

#[sqlx::test]
async fn null_clears_the_nullable_settings(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    patch_settings(
        &app,
        &token,
        json!({ "hourly_wage": 30000, "base_work_hours": 8, "monthly_budget": 100 }),
    )
    .await;

    let response = patch_settings(
        &app,
        &token,
        json!({ "hourly_wage": null, "base_work_hours": null, "monthly_budget": null }),
    )
    .await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["hourly_wage"], Value::Null);
    assert_eq!(response.body["base_work_hours"], Value::Null);
    assert_eq!(response.body["monthly_budget"], Value::Null);
}

#[sqlx::test]
async fn null_for_a_pomodoro_field_is_a_malformed_request(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = patch_settings(&app, &token, json!({ "focus_minutes": null })).await;

    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&response), "invalid_request");
}

#[sqlx::test]
async fn out_of_range_values_are_rejected(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let cases = [
        ("hourly_wage", json!(0)),
        ("hourly_wage", json!(-5)),
        ("hourly_wage", json!(1_000_000_000_001_i64)),
        ("monthly_budget", json!(0)),
        ("monthly_budget", json!(-1)),
        ("monthly_budget", json!(1_000_000_000_001_i64)),
        ("base_work_hours", json!(0)),
        ("base_work_hours", json!(-1)),
        ("base_work_hours", json!(24.01)),
        ("base_work_hours", json!(7.255)),
        ("focus_minutes", json!(0)),
        ("focus_minutes", json!(241)),
        ("short_break_minutes", json!(0)),
        ("long_break_minutes", json!(241)),
        ("long_break_every", json!(0)),
        ("long_break_every", json!(13)),
    ];
    for (field, value) in cases {
        let response = patch_settings(&app, &token, json!({ field: value })).await;

        assert_eq!(
            response.status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{field} = {value}"
        );
        assert_eq!(error_code(&response), "validation_failed");
        assert_eq!(
            field_codes(&response, field),
            ["range"],
            "{field} = {value}"
        );
    }
}

#[sqlx::test]
async fn boundary_values_are_accepted(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = patch_settings(
        &app,
        &token,
        json!({
            "hourly_wage": 1_000_000_000_000_i64,
            "monthly_budget": 1,
            "base_work_hours": 24,
            "focus_minutes": 240,
            "short_break_minutes": 1,
            "long_break_minutes": 240,
            "long_break_every": 12,
        }),
    )
    .await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["hourly_wage"], 1_000_000_000_000_i64);
    assert_eq!(response.body["base_work_hours"], 24.0);
}

#[sqlx::test]
async fn wrong_types_are_a_malformed_request(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    for body in [
        json!({ "hourly_wage": "30000" }),
        json!({ "hourly_wage": 1.5 }),
        json!({ "focus_minutes": 25.5 }),
    ] {
        let response = patch_settings(&app, &token, body).await;

        assert_eq!(response.status, StatusCode::BAD_REQUEST);
    }
}

#[sqlx::test]
async fn one_users_settings_do_not_leak_to_another(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;

    patch_settings(&app, &ada, json!({ "hourly_wage": 30000 })).await;

    assert_eq!(
        get_settings(&app, &grace).await.body["hourly_wage"],
        Value::Null
    );
}

#[sqlx::test]
async fn settings_require_authentication(db: PgPool) {
    let app = common::app(db);

    let get = TestRequest::get("/api/v1/settings").send(&app).await;
    let patch = TestRequest::new(Method::PATCH, "/api/v1/settings")
        .json(&json!({}))
        .send(&app)
        .await;

    assert_eq!(get.status, StatusCode::UNAUTHORIZED);
    assert_eq!(patch.status, StatusCode::UNAUTHORIZED);
}
