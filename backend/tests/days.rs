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

// ---------------------------------------------------------------------------------------------
// Day mode
// ---------------------------------------------------------------------------------------------

async fn put_mode(app: &Router, token: &str, date: NaiveDate, mode: &str) -> TestResponse {
    TestRequest::new(Method::PUT, &format!("/api/v1/days/{date}/mode"))
        .bearer(token)
        .json(&json!({ "mode": mode }))
        .send(app)
        .await
}

async fn uncomplete(app: &Router, token: &str, habit_id: &str, date: NaiveDate) {
    let response = TestRequest::new(
        Method::DELETE,
        &format!("/api/v1/habits/{habit_id}/completions/{date}"),
    )
    .bearer(token)
    .send(app)
    .await;
    assert_eq!(response.status, StatusCode::NO_CONTENT);
}

#[sqlx::test]
async fn day_mode_is_normal_and_unlocked_by_default(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let today = today(&db).await;

    let day = get_day(&app, &token, today).await;

    assert_eq!(day.body["mode"], "normal");
    assert_eq!(day.body["mode_locked"], false);
}

#[sqlx::test]
async fn day_mode_can_be_set_to_low_energy_and_back(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let today = today(&db).await;

    let set = put_mode(&app, &token, today, "low_energy").await;
    let low = get_day(&app, &token, today).await;
    let reset = put_mode(&app, &token, today, "normal").await;
    let normal = get_day(&app, &token, today).await;

    assert_eq!(set.status, StatusCode::NO_CONTENT);
    assert_eq!(low.body["mode"], "low_energy");
    assert_eq!(low.body["mode_locked"], false);
    assert_eq!(reset.status, StatusCode::NO_CONTENT);
    assert_eq!(normal.body["mode"], "normal");
}

#[sqlx::test]
async fn setting_the_same_mode_twice_succeeds(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let today = today(&db).await;

    let normal = put_mode(&app, &token, today, "normal").await;
    put_mode(&app, &token, today, "low_energy").await;
    let low_again = put_mode(&app, &token, today, "low_energy").await;

    assert_eq!(normal.status, StatusCode::NO_CONTENT);
    assert_eq!(low_again.status, StatusCode::NO_CONTENT);
    assert_eq!(
        get_day(&app, &token, today).await.body["mode"],
        "low_energy"
    );
}

#[sqlx::test]
async fn day_mode_cannot_be_set_for_another_day(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let today = today(&db).await;
    let yesterday = today.checked_sub_days(Days::new(1)).unwrap();
    let tomorrow = today.succ_opt().unwrap();

    for date in [yesterday, tomorrow] {
        let response = put_mode(&app, &token, date, "low_energy").await;

        assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            response.body["error"]["details"]["fields"]["date"],
            json!(["not_today"])
        );
        assert_eq!(get_day(&app, &token, date).await.body["mode"], "normal");
    }
}

#[sqlx::test]
async fn day_mode_is_locked_for_days_other_than_today(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let today = today(&db).await;
    let yesterday = today.checked_sub_days(Days::new(1)).unwrap();

    let past = get_day(&app, &token, yesterday).await;
    let future = get_day(&app, &token, today.succ_opt().unwrap()).await;

    assert_eq!(past.body["mode_locked"], true);
    assert_eq!(future.body["mode_locked"], true);
}

#[sqlx::test]
async fn day_mode_is_locked_by_a_completion_and_unlocked_by_removing_it(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let today = today(&db).await;
    let habit = add_habit(&app, &token, json!({ "name": "Read" })).await;
    complete(&app, &token, &habit, today).await;

    let locked_view = get_day(&app, &token, today).await;
    let rejected = put_mode(&app, &token, today, "low_energy").await;
    uncomplete(&app, &token, &habit, today).await;
    let unlocked_view = get_day(&app, &token, today).await;
    let accepted = put_mode(&app, &token, today, "low_energy").await;

    assert_eq!(locked_view.body["mode_locked"], true);
    assert_eq!(rejected.status, StatusCode::CONFLICT);
    assert_eq!(rejected.body["error"]["code"], "day_mode_locked");
    assert_eq!(unlocked_view.body["mode_locked"], false);
    assert_eq!(accepted.status, StatusCode::NO_CONTENT);
}

#[sqlx::test]
async fn day_mode_is_not_locked_by_a_completion_of_an_archived_habit(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let today = today(&db).await;
    let habit = add_habit(&app, &token, json!({ "name": "Old" })).await;
    complete(&app, &token, &habit, today).await;
    TestRequest::new(Method::PATCH, &format!("/api/v1/habits/{habit}"))
        .bearer(&token)
        .json(&json!({ "archived": true }))
        .send(&app)
        .await;

    let view = get_day(&app, &token, today).await;
    let to_low_energy = put_mode(&app, &token, today, "low_energy").await;
    let to_normal = put_mode(&app, &token, today, "normal").await;

    assert_eq!(view.body["mode_locked"], false);
    assert_eq!(to_low_energy.status, StatusCode::NO_CONTENT);
    assert_eq!(to_normal.status, StatusCode::NO_CONTENT);
}

#[sqlx::test]
async fn day_mode_cannot_return_to_normal_while_a_kept_habit_is_checked(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let today = today(&db).await;
    let habit = add_habit(&app, &token, json!({ "name": "Read", "low_energy": true })).await;
    put_mode(&app, &token, today, "low_energy").await;
    complete(&app, &token, &habit, today).await;

    let response = put_mode(&app, &token, today, "normal").await;

    assert_eq!(response.status, StatusCode::CONFLICT);
    assert_eq!(response.body["error"]["code"], "day_mode_locked");
    assert_eq!(
        get_day(&app, &token, today).await.body["mode"],
        "low_energy"
    );
}

#[sqlx::test]
async fn setting_the_current_mode_succeeds_even_when_locked(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let today = today(&db).await;
    let habit = add_habit(&app, &token, json!({ "name": "Read" })).await;
    complete(&app, &token, &habit, today).await;

    let response = put_mode(&app, &token, today, "normal").await;

    assert_eq!(response.status, StatusCode::NO_CONTENT);
}

#[sqlx::test]
async fn low_energy_day_shows_only_kept_habits_under_their_easier_names(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let today = today(&db).await;
    add_habit(&app, &token, json!({ "name": "Hidden" })).await;
    add_habit(
        &app,
        &token,
        json!({ "name": "Run 5 km", "low_energy": true, "low_energy_name": "Walk 5 minutes" }),
    )
    .await;
    add_habit(&app, &token, json!({ "name": "Read", "low_energy": true })).await;

    let normal = get_day(&app, &token, today).await;
    put_mode(&app, &token, today, "low_energy").await;
    let low = get_day(&app, &token, today).await;

    assert_eq!(
        names(&normal, "habits", "name"),
        ["Hidden", "Run 5 km", "Read"]
    );
    assert_eq!(names(&low, "habits", "name"), ["Walk 5 minutes", "Read"]);
}

#[sqlx::test]
async fn day_mode_is_not_visible_to_another_user(db: PgPool) {
    let app = common::app(db.clone());
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let today = today(&db).await;
    put_mode(&app, &ada, today, "low_energy").await;

    let day = get_day(&app, &grace, today).await;

    assert_eq!(day.body["mode"], "normal");
}

#[sqlx::test]
async fn day_mode_rejects_an_unknown_mode(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let today = today(&db).await;

    let response = put_mode(&app, &token, today, "sleepy").await;

    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(response.body["error"]["code"], "invalid_request");
}

#[sqlx::test]
async fn day_mode_requires_authentication(db: PgPool) {
    let app = common::app(db);

    let response = TestRequest::new(Method::PUT, "/api/v1/days/2026-11-12/mode")
        .json(&json!({ "mode": "normal" }))
        .send(&app)
        .await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
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

async fn set_settings(app: &Router, token: &str, body: Value) {
    let response = TestRequest::new(Method::PATCH, "/api/v1/settings")
        .bearer(token)
        .json(&body)
        .send(app)
        .await;
    assert_eq!(response.status, StatusCode::OK);
}

async fn log_work(app: &Router, token: &str, date: NaiveDate, hours: f64) {
    let response = TestRequest::new(Method::PUT, &format!("/api/v1/work-entries/{date}"))
        .bearer(token)
        .json(&json!({ "hours": hours }))
        .send(app)
        .await;
    assert_eq!(response.status, StatusCode::OK);
}

async fn add_category(app: &Router, token: &str, kind: &str) -> String {
    let response = TestRequest::post("/api/v1/finance-categories")
        .bearer(token)
        .json(&json!({ "kind": kind, "name": kind, "color": "#22C55E" }))
        .send(app)
        .await;
    assert_eq!(response.status, StatusCode::CREATED);
    response.body["id"].as_str().unwrap().to_owned()
}

async fn add_transaction(app: &Router, token: &str, category: &str, amount: i64, date: NaiveDate) {
    let response = TestRequest::post("/api/v1/transactions")
        .bearer(token)
        .json(&json!({ "category_id": category, "amount": amount, "date": date }))
        .send(app)
        .await;
    assert_eq!(response.status, StatusCode::CREATED);
}

#[sqlx::test]
async fn day_has_no_work_and_no_expenses_by_default(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;

    let day = get_day(&app, &token, today(&db).await).await;

    assert_eq!(day.body["work"], Value::Null);
    assert_eq!(day.body["expense_total"], 0);
}

#[sqlx::test]
async fn day_shows_the_hours_and_pay_of_its_work_entry(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    set_settings(
        &app,
        &token,
        json!({ "hourly_wage": 30_001, "base_work_hours": 8 }),
    )
    .await;
    let today = today(&db).await;
    log_work(&app, &token, today, 1.25).await;

    let day = get_day(&app, &token, today).await;
    let yesterday = get_day(&app, &token, today.pred_opt().unwrap()).await;

    // 1.25 x 30,001 = 37,501.25, rounded half up.
    assert_eq!(day.body["work"], json!({ "hours": 1.25, "pay": 37_501 }));
    assert_eq!(yesterday.body["work"], Value::Null);
}

#[sqlx::test]
async fn day_totals_only_the_expenses_of_that_date(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let today = today(&db).await;
    let expense = add_category(&app, &token, "expense").await;
    let income = add_category(&app, &token, "income").await;
    add_transaction(&app, &token, &expense, 12_000, today).await;
    add_transaction(&app, &token, &expense, 3_500, today).await;
    add_transaction(&app, &token, &expense, 99_000, today.pred_opt().unwrap()).await;
    add_transaction(&app, &token, &income, 500_000, today).await;

    let day = get_day(&app, &token, today).await;

    assert_eq!(day.body["expense_total"], 15_500);
}

#[sqlx::test]
async fn day_money_is_not_visible_to_another_user(db: PgPool) {
    let app = common::app(db.clone());
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let today = today(&db).await;
    set_settings(
        &app,
        &ada,
        json!({ "hourly_wage": 30_000, "base_work_hours": 8 }),
    )
    .await;
    log_work(&app, &ada, today, 8.0).await;
    let expense = add_category(&app, &ada, "expense").await;
    add_transaction(&app, &ada, &expense, 10_000, today).await;

    let day = get_day(&app, &grace, today).await;

    assert_eq!(day.body["work"], Value::Null);
    assert_eq!(day.body["expense_total"], 0);
}

/// Writes a study session directly, so that a session can be given any status and duration.
async fn add_session(db: &PgPool, email: &str, status: &str, date: NaiveDate, seconds: i32) {
    sqlx::query(
        "INSERT INTO study_sessions
             (user_id, status, planned_minutes, started_at, ended_at, duration_seconds, local_date)
         SELECT id, $2, 25, now(), now(), $3, $4 FROM users WHERE email = $1",
    )
    .bind(email)
    .bind(status)
    .bind(seconds)
    .bind(date)
    .execute(db)
    .await
    .unwrap();
}

#[sqlx::test]
async fn day_has_no_study_time_by_default(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;

    let day = get_day(&app, &token, today(&db).await).await;

    assert_eq!(day.body["study_seconds"], 0);
}

#[sqlx::test]
async fn day_sums_only_completed_sessions_of_that_date(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let today = today(&db).await;
    add_session(&db, ADA, "completed", today, 1_500).await;
    add_session(&db, ADA, "completed", today, 900).await;
    add_session(&db, ADA, "cancelled", today, 700).await;
    add_session(&db, ADA, "completed", today.pred_opt().unwrap(), 3_000).await;

    let day = get_day(&app, &token, today).await;

    assert_eq!(day.body["study_seconds"], 2_400);
}

#[sqlx::test]
async fn day_study_time_is_not_visible_to_another_user(db: PgPool) {
    let app = common::app(db.clone());
    sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let today = today(&db).await;
    add_session(&db, ADA, "completed", today, 1_500).await;

    let day = get_day(&app, &grace, today).await;

    assert_eq!(day.body["study_seconds"], 0);
}
