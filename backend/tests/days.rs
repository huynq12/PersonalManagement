//! Integration tests for the day view: the scheduled habits and the to-dos of one date.

// Test helpers are not `#[test]` functions, so the clippy.toml test exemption does not reach them.
#![allow(clippy::unwrap_used)]

use axum::{
    Router,
    http::{Method, StatusCode},
};
use chrono::{Datelike, Days, NaiveDate};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::{TestRequest, TestResponse, sign_up, today};

mod common;

const ADA: &str = "ada@example.com";
const GRACE: &str = "grace@example.com";

async fn get_day(app: &Router, token: &str, date: NaiveDate) -> TestResponse {
    TestRequest::get(&format!("/api/v1/days/{date}"))
        .bearer(token)
        .send(app)
        .await
}

/// Creates a habit and returns its ID.
async fn add_habit(app: &Router, token: &str, body: Value) -> String {
    let response = TestRequest::post("/api/v1/habits")
        .bearer(token)
        .json(&body)
        .send(app)
        .await;
    assert_eq!(response.status, StatusCode::CREATED);
    response.body["id"].as_str().unwrap().to_owned()
}

async fn add_task(app: &Router, token: &str, date: NaiveDate, title: &str) {
    let response = TestRequest::post("/api/v1/tasks")
        .bearer(token)
        .json(&json!({ "date": date, "title": title }))
        .send(app)
        .await;
    assert_eq!(response.status, StatusCode::CREATED);
}

async fn complete(app: &Router, token: &str, habit_id: &str, date: NaiveDate) {
    let response = TestRequest::new(
        Method::PUT,
        &format!("/api/v1/habits/{habit_id}/completions/{date}"),
    )
    .bearer(token)
    .send(app)
    .await;
    assert_eq!(response.status, StatusCode::NO_CONTENT);
}

/// ISO weekday number: 1 (Monday) to 7 (Sunday).
fn weekday(date: NaiveDate) -> u32 {
    date.weekday().number_from_monday()
}

fn names<'a>(day: &'a TestResponse, list: &str, field: &str) -> Vec<&'a str> {
    day.body[list]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item[field].as_str().unwrap())
        .collect()
}

#[sqlx::test]
async fn day_is_empty_for_a_new_user(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let today = today(&db).await;

    let day = get_day(&app, &token, today).await;

    assert_eq!(day.status, StatusCode::OK);
    assert_eq!(day.body["date"], today.to_string());
    assert_eq!(day.body["habits"], json!([]));
    assert_eq!(day.body["tasks"], json!([]));
}

#[sqlx::test]
async fn day_lists_habits_scheduled_for_that_weekday_in_order(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let today = today(&db).await;
    let tomorrow = today.succ_opt().unwrap();
    add_habit(&app, &token, json!({ "name": "Daily" })).await;
    add_habit(
        &app,
        &token,
        json!({ "name": "Today only", "weekdays": [weekday(today)] }),
    )
    .await;
    add_habit(
        &app,
        &token,
        json!({ "name": "Tomorrow only", "weekdays": [weekday(tomorrow)] }),
    )
    .await;

    let day = get_day(&app, &token, today).await;

    assert_eq!(names(&day, "habits", "name"), ["Daily", "Today only"]);
}

#[sqlx::test]
async fn day_reports_whether_each_habit_is_completed_on_that_date(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let today = today(&db).await;
    let yesterday = today.checked_sub_days(Days::new(1)).unwrap();
    let done = add_habit(&app, &token, json!({ "name": "Done" })).await;
    add_habit(&app, &token, json!({ "name": "Open" })).await;
    complete(&app, &token, &done, today).await;

    let today_view = get_day(&app, &token, today).await;
    let yesterday_view = get_day(&app, &token, yesterday).await;

    assert_eq!(today_view.body["habits"][0]["id"], done);
    assert_eq!(today_view.body["habits"][0]["completed"], true);
    assert_eq!(today_view.body["habits"][1]["completed"], false);
    assert_eq!(yesterday_view.body["habits"][0]["completed"], false);
}

#[sqlx::test]
async fn day_hides_archived_habits(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let today = today(&db).await;
    let id = add_habit(&app, &token, json!({ "name": "Old" })).await;
    TestRequest::new(Method::PATCH, &format!("/api/v1/habits/{id}"))
        .bearer(&token)
        .json(&json!({ "archived": true }))
        .send(&app)
        .await;

    let day = get_day(&app, &token, today).await;

    assert_eq!(day.body["habits"], json!([]));
}

#[sqlx::test]
async fn day_lists_the_tasks_of_that_date_only(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let today = today(&db).await;
    let tomorrow = today.succ_opt().unwrap();
    add_task(&app, &token, today, "Buy milk").await;
    add_task(&app, &token, today, "Call mom").await;
    add_task(&app, &token, tomorrow, "Plan trip").await;

    let day = get_day(&app, &token, today).await;
    let next_day = get_day(&app, &token, tomorrow).await;

    assert_eq!(names(&day, "tasks", "title"), ["Buy milk", "Call mom"]);
    assert_eq!(day.body["tasks"][0]["is_done"], false);
    assert_eq!(names(&next_day, "tasks", "title"), ["Plan trip"]);
}

#[sqlx::test]
async fn day_shows_only_the_data_of_the_signed_in_user(db: PgPool) {
    let app = common::app(db.clone());
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let today = today(&db).await;
    add_habit(&app, &ada, json!({ "name": "Read" })).await;
    add_task(&app, &ada, today, "Buy milk").await;

    let day = get_day(&app, &grace, today).await;

    assert_eq!(day.body["habits"], json!([]));
    assert_eq!(day.body["tasks"], json!([]));
}

#[sqlx::test]
async fn day_rejects_malformed_date(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = TestRequest::get("/api/v1/days/today")
        .bearer(&token)
        .send(&app)
        .await;

    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(response.body["error"]["code"], "invalid_request");
}

#[sqlx::test]
async fn day_requires_authentication(db: PgPool) {
    let app = common::app(db);

    let response = TestRequest::get("/api/v1/days/2026-11-12").send(&app).await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
    assert_eq!(response.body["error"]["code"], "unauthorized");
}
