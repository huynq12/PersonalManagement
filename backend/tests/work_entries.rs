//! Integration tests for the `/work-entries` endpoints.

// Test helpers are not `#[test]` functions, so the clippy.toml test exemption does not reach them.
#![allow(clippy::unwrap_used)]

use axum::{
    Router,
    http::{Method, StatusCode},
};
use chrono::Days;
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::{TestRequest, TestResponse, sign_up, today};

mod common;

const ADA: &str = "ada@example.com";
const GRACE: &str = "grace@example.com";

async fn set_settings(app: &Router, token: &str, body: Value) {
    let response = TestRequest::new(Method::PATCH, "/api/v1/settings")
        .bearer(token)
        .json(&body)
        .send(app)
        .await;
    assert_eq!(response.status, StatusCode::OK);
}

/// Signs up, sets the wage and base hours, and returns the token.
async fn user_with_wage(app: &Router, email: &str, wage: i64, base_hours: f64) -> String {
    let token = sign_up(app, email).await;
    set_settings(
        app,
        &token,
        json!({ "hourly_wage": wage, "base_work_hours": base_hours }),
    )
    .await;
    token
}

async fn put_entry(app: &Router, token: &str, day: &str, body: Value) -> TestResponse {
    TestRequest::new(Method::PUT, &format!("/api/v1/work-entries/{day}"))
        .bearer(token)
        .json(&body)
        .send(app)
        .await
}

async fn list(app: &Router, token: &str, month: &str) -> TestResponse {
    TestRequest::get(&format!("/api/v1/work-entries?month={month}"))
        .bearer(token)
        .send(app)
        .await
}

async fn delete_entry(app: &Router, token: &str, day: &str) -> TestResponse {
    TestRequest::new(Method::DELETE, &format!("/api/v1/work-entries/{day}"))
        .bearer(token)
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
async fn create_computes_the_pay(db: PgPool) {
    let app = common::app(db);
    let token = user_with_wage(&app, ADA, 30_000, 8.0).await;

    let response = put_entry(
        &app,
        &token,
        "2026-01-10",
        json!({ "hours": 7.5, "note": "Inventory" }),
    )
    .await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(
        response.body,
        json!({
            "date": "2026-01-10",
            "hours": 7.5,
            "hourly_wage": 30_000,
            "base_hours": 8.0,
            "pay": 225_000,
            "comparison": "below",
            "note": "Inventory",
        })
    );
}

#[sqlx::test]
async fn pay_rounds_half_up(db: PgPool) {
    let app = common::app(db);
    let token = user_with_wage(&app, ADA, 30_001, 8.0).await;

    // 1.25 * 30001 = 37501.25 -> 37501; 0.5 * 30001 = 15000.5 -> 15001.
    let first = put_entry(&app, &token, "2026-01-10", json!({ "hours": 1.25 })).await;
    let second = put_entry(&app, &token, "2026-01-11", json!({ "hours": 0.5 })).await;

    assert_eq!(first.body["pay"], 37_501);
    assert_eq!(second.body["pay"], 15_001);
}

#[sqlx::test]
async fn comparison_is_above_equal_or_below_the_base_hours(db: PgPool) {
    let app = common::app(db);
    let token = user_with_wage(&app, ADA, 10_000, 8.0).await;

    let above = put_entry(&app, &token, "2026-01-10", json!({ "hours": 8.25 })).await;
    let equal = put_entry(&app, &token, "2026-01-11", json!({ "hours": 8 })).await;
    let below = put_entry(&app, &token, "2026-01-12", json!({ "hours": 7.99 })).await;

    assert_eq!(above.body["comparison"], "above");
    assert_eq!(equal.body["comparison"], "equal");
    assert_eq!(below.body["comparison"], "below");
}

#[sqlx::test]
async fn the_note_is_optional_and_a_blank_note_is_stored_as_null(db: PgPool) {
    let app = common::app(db);
    let token = user_with_wage(&app, ADA, 10_000, 8.0).await;

    let without = put_entry(&app, &token, "2026-01-10", json!({ "hours": 2 })).await;
    let blank = put_entry(
        &app,
        &token,
        "2026-01-11",
        json!({ "hours": 2, "note": "  " }),
    )
    .await;

    assert_eq!(without.body["note"], Value::Null);
    assert_eq!(blank.body["note"], Value::Null);
}

#[sqlx::test]
async fn update_changes_hours_and_note_and_keeps_the_copied_wage(db: PgPool) {
    let app = common::app(db);
    let token = user_with_wage(&app, ADA, 30_000, 8.0).await;
    put_entry(
        &app,
        &token,
        "2026-01-10",
        json!({ "hours": 4, "note": "old" }),
    )
    .await;
    set_settings(
        &app,
        &token,
        json!({ "hourly_wage": 50_000, "base_work_hours": 6 }),
    )
    .await;

    let response = put_entry(&app, &token, "2026-01-10", json!({ "hours": 7 })).await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["hours"], 7.0);
    assert_eq!(response.body["note"], Value::Null);
    assert_eq!(response.body["hourly_wage"], 30_000);
    assert_eq!(response.body["base_hours"], 8.0);
    assert_eq!(response.body["pay"], 210_000);
    assert_eq!(response.body["comparison"], "below");
    let month = list(&app, &token, "2026-01").await;
    assert_eq!(month.body["entries"].as_array().unwrap().len(), 1);
}

#[sqlx::test]
async fn update_works_after_the_settings_were_cleared(db: PgPool) {
    let app = common::app(db);
    let token = user_with_wage(&app, ADA, 30_000, 8.0).await;
    put_entry(&app, &token, "2026-01-10", json!({ "hours": 4 })).await;
    set_settings(
        &app,
        &token,
        json!({ "hourly_wage": null, "base_work_hours": null }),
    )
    .await;

    let response = put_entry(&app, &token, "2026-01-10", json!({ "hours": 5 })).await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["pay"], 150_000);
}

#[sqlx::test]
async fn a_new_entry_after_a_settings_change_uses_the_new_wage(db: PgPool) {
    let app = common::app(db);
    let token = user_with_wage(&app, ADA, 30_000, 8.0).await;
    put_entry(&app, &token, "2026-01-10", json!({ "hours": 4 })).await;
    set_settings(
        &app,
        &token,
        json!({ "hourly_wage": 50_000, "base_work_hours": 6 }),
    )
    .await;

    let response = put_entry(&app, &token, "2026-01-11", json!({ "hours": 4 })).await;

    assert_eq!(response.body["hourly_wage"], 50_000);
    assert_eq!(response.body["base_hours"], 6.0);
    assert_eq!(response.body["pay"], 200_000);
    let month = list(&app, &token, "2026-01").await;
    let entries = month.body["entries"].as_array().unwrap();
    assert_eq!(entries[0]["hourly_wage"], 30_000);
    assert_eq!(entries[1]["hourly_wage"], 50_000);
}

#[sqlx::test]
async fn creating_without_work_settings_is_a_409(db: PgPool) {
    let app = common::app(db);
    let no_settings = sign_up(&app, ADA).await;
    let only_wage = sign_up(&app, GRACE).await;
    set_settings(&app, &only_wage, json!({ "hourly_wage": 30_000 })).await;

    let first = put_entry(&app, &no_settings, "2026-01-10", json!({ "hours": 4 })).await;
    let second = put_entry(&app, &only_wage, "2026-01-10", json!({ "hours": 4 })).await;

    for response in [first, second] {
        assert_eq!(response.status, StatusCode::CONFLICT);
        assert_eq!(error_code(&response), "work_settings_missing");
    }
}

#[sqlx::test]
async fn a_future_date_is_a_422(db: PgPool) {
    let app = common::app(db.clone());
    let token = user_with_wage(&app, ADA, 30_000, 8.0).await;
    let tomorrow = today(&db).await + Days::new(1);

    let response = put_entry(&app, &token, &tomorrow.to_string(), json!({ "hours": 4 })).await;
    let today_entry = put_entry(
        &app,
        &token,
        &today(&db).await.to_string(),
        json!({ "hours": 4 }),
    )
    .await;

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(error_code(&response), "validation_failed");
    assert_eq!(field_codes(&response, "date"), ["future"]);
    assert_eq!(today_entry.status, StatusCode::OK);
}

#[sqlx::test]
async fn bad_hours_are_a_422(db: PgPool) {
    let app = common::app(db);
    let token = user_with_wage(&app, ADA, 30_000, 8.0).await;

    for hours in [json!(0), json!(-1), json!(24.01), json!(7.255)] {
        let response = put_entry(&app, &token, "2026-01-10", json!({ "hours": hours })).await;

        assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY, "{hours}");
        assert_eq!(field_codes(&response, "hours"), ["range"], "{hours}");
    }
    let limit = put_entry(&app, &token, "2026-01-10", json!({ "hours": 24 })).await;
    assert_eq!(limit.status, StatusCode::OK);
}

#[sqlx::test]
async fn a_missing_or_non_numeric_hours_value_is_a_400(db: PgPool) {
    let app = common::app(db);
    let token = user_with_wage(&app, ADA, 30_000, 8.0).await;

    let missing = put_entry(&app, &token, "2026-01-10", json!({})).await;
    let text = put_entry(&app, &token, "2026-01-10", json!({ "hours": "8" })).await;
    let bad_date = put_entry(&app, &token, "2026-02-30", json!({ "hours": 8 })).await;

    for response in [missing, text, bad_date] {
        assert_eq!(response.status, StatusCode::BAD_REQUEST);
        assert_eq!(error_code(&response), "invalid_request");
    }
}

#[sqlx::test]
async fn an_over_long_note_is_a_422(db: PgPool) {
    let app = common::app(db);
    let token = user_with_wage(&app, ADA, 30_000, 8.0).await;

    let response = put_entry(
        &app,
        &token,
        "2026-01-10",
        json!({ "hours": 4, "note": "x".repeat(501) }),
    )
    .await;

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(field_codes(&response, "note"), ["length"]);
}

#[sqlx::test]
async fn the_month_filter_excludes_neighbouring_months(db: PgPool) {
    let app = common::app(db);
    let token = user_with_wage(&app, ADA, 10_000, 8.0).await;
    for day in ["2025-12-31", "2026-01-01", "2026-01-31", "2026-02-01"] {
        put_entry(&app, &token, day, json!({ "hours": 1 })).await;
    }

    let january = list(&app, &token, "2026-01").await;

    assert_eq!(january.status, StatusCode::OK);
    let days: Vec<&str> = january.body["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["date"].as_str().unwrap())
        .collect();
    assert_eq!(days, ["2026-01-01", "2026-01-31"]);
}

#[sqlx::test]
async fn the_summary_totals_the_month(db: PgPool) {
    let app = common::app(db);
    let token = user_with_wage(&app, ADA, 30_001, 8.0).await;
    put_entry(&app, &token, "2026-01-10", json!({ "hours": 1.25 })).await;
    put_entry(&app, &token, "2026-01-11", json!({ "hours": 8 })).await;
    put_entry(&app, &token, "2026-02-01", json!({ "hours": 3 })).await;

    let response = list(&app, &token, "2026-01").await;

    // 37501 + 240008; the total is the sum of the daily pays, not a re-rounded product.
    assert_eq!(
        response.body["summary"],
        json!({ "total_hours": 9.25, "total_pay": 277_509, "days_worked": 2 })
    );
}

#[sqlx::test]
async fn an_empty_month_has_a_zero_summary(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = list(&app, &token, "2026-01").await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(
        response.body,
        json!({
            "entries": [],
            "summary": { "total_hours": 0.0, "total_pay": 0, "days_worked": 0 },
        })
    );
}

#[sqlx::test]
async fn a_malformed_or_missing_month_is_a_422(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    for month in ["2026-13", "2026-1", "26-01", "abc", "2026-01-01", "2026-00"] {
        let response = list(&app, &token, month).await;

        assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY, "{month}");
        assert_eq!(field_codes(&response, "month"), ["format"], "{month}");
    }
    let missing = TestRequest::get("/api/v1/work-entries")
        .bearer(&token)
        .send(&app)
        .await;
    assert_eq!(missing.status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn delete_removes_the_entry(db: PgPool) {
    let app = common::app(db);
    let token = user_with_wage(&app, ADA, 10_000, 8.0).await;
    put_entry(&app, &token, "2026-01-10", json!({ "hours": 4 })).await;

    let deleted = delete_entry(&app, &token, "2026-01-10").await;
    let again = delete_entry(&app, &token, "2026-01-10").await;

    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    assert_eq!(again.status, StatusCode::NOT_FOUND);
    assert_eq!(
        list(&app, &token, "2026-01").await.body["summary"]["days_worked"],
        0
    );
}

#[sqlx::test]
async fn another_users_entries_are_invisible(db: PgPool) {
    let app = common::app(db);
    let ada = user_with_wage(&app, ADA, 10_000, 8.0).await;
    let grace = user_with_wage(&app, GRACE, 20_000, 8.0).await;
    put_entry(&app, &ada, "2026-01-10", json!({ "hours": 4 })).await;

    let listed = list(&app, &grace, "2026-01").await;
    let deleted = delete_entry(&app, &grace, "2026-01-10").await;
    let own = put_entry(&app, &grace, "2026-01-10", json!({ "hours": 2 })).await;

    assert_eq!(listed.body["entries"], json!([]));
    assert_eq!(deleted.status, StatusCode::NOT_FOUND);
    assert_eq!(own.body["hourly_wage"], 20_000);
    let hers = list(&app, &ada, "2026-01").await;
    assert_eq!(hers.body["entries"][0]["hours"], 4.0);
    assert_eq!(hers.body["entries"][0]["hourly_wage"], 10_000);
}

#[sqlx::test]
async fn endpoints_require_authentication(db: PgPool) {
    let app = common::app(db);

    let get = TestRequest::get("/api/v1/work-entries?month=2026-01")
        .send(&app)
        .await;
    let put = TestRequest::new(Method::PUT, "/api/v1/work-entries/2026-01-10")
        .json(&json!({ "hours": 1 }))
        .send(&app)
        .await;
    let delete = TestRequest::new(Method::DELETE, "/api/v1/work-entries/2026-01-10")
        .send(&app)
        .await;

    for response in [get, put, delete] {
        assert_eq!(response.status, StatusCode::UNAUTHORIZED);
    }
}

#[sqlx::test]
async fn concurrent_creates_leave_one_row(db: PgPool) {
    let app = common::app(db.clone());
    let token = user_with_wage(&app, ADA, 10_000, 8.0).await;

    let (a, b) = tokio::join!(
        put_entry(&app, &token, "2026-01-10", json!({ "hours": 2 })),
        put_entry(&app, &token, "2026-01-10", json!({ "hours": 3 })),
    );

    assert_eq!(a.status, StatusCode::OK);
    assert_eq!(b.status, StatusCode::OK);
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM work_entries")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(count, 1);
}
