//! Integration tests for habits, their completions, and streaks.

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

async fn create_habit(app: &Router, token: &str, body: Value) -> TestResponse {
    TestRequest::post("/api/v1/habits")
        .bearer(token)
        .json(&body)
        .send(app)
        .await
}

/// Creates a habit scheduled every day and returns its ID.
async fn create_daily_habit(app: &Router, token: &str, name: &str) -> String {
    let response = create_habit(app, token, json!({ "name": name })).await;
    assert_eq!(response.status, StatusCode::CREATED);
    response.body["id"].as_str().unwrap().to_owned()
}

async fn list_habits(app: &Router, token: &str, query: &str) -> TestResponse {
    TestRequest::get(&format!("/api/v1/habits{query}"))
        .bearer(token)
        .send(app)
        .await
}

async fn patch_habit(app: &Router, token: &str, id: &str, body: Value) -> TestResponse {
    TestRequest::new(Method::PATCH, &format!("/api/v1/habits/{id}"))
        .bearer(token)
        .json(&body)
        .send(app)
        .await
}

async fn complete(app: &Router, token: &str, id: &str, date: NaiveDate) -> TestResponse {
    TestRequest::new(
        Method::PUT,
        &format!("/api/v1/habits/{id}/completions/{date}"),
    )
    .bearer(token)
    .send(app)
    .await
}

async fn uncomplete(app: &Router, token: &str, id: &str, date: NaiveDate) -> TestResponse {
    TestRequest::new(
        Method::DELETE,
        &format!("/api/v1/habits/{id}/completions/{date}"),
    )
    .bearer(token)
    .send(app)
    .await
}

/// The current streak of the only habit the user has.
async fn only_streak(app: &Router, token: &str) -> i64 {
    let habits = list_habits(app, token, "").await;
    assert_eq!(habits.body.as_array().unwrap().len(), 1);
    habits.body[0]["current_streak"].as_i64().unwrap()
}

fn days_ago(today: NaiveDate, days: u64) -> NaiveDate {
    today.checked_sub_days(Days::new(days)).unwrap()
}

/// ISO weekday number: 1 (Monday) to 7 (Sunday).
fn weekday(date: NaiveDate) -> u32 {
    date.weekday().number_from_monday()
}

fn names(response: &TestResponse) -> Vec<&str> {
    response
        .body
        .as_array()
        .unwrap()
        .iter()
        .map(|habit| habit["name"].as_str().unwrap())
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Create
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn create_defaults_to_every_weekday(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = create_habit(&app, &token, json!({ "name": "Read" })).await;

    assert_eq!(response.status, StatusCode::CREATED);
    assert_eq!(response.body["name"], "Read");
    assert_eq!(response.body["weekdays"], json!([1, 2, 3, 4, 5, 6, 7]));
    assert_eq!(response.body["archived_at"], Value::Null);
    assert_eq!(response.body["current_streak"], 0);
    assert!(response.body["id"].is_string());
}

#[sqlx::test]
async fn create_stores_chosen_weekdays_sorted_without_duplicates(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = create_habit(
        &app,
        &token,
        json!({ "name": "Gym", "weekdays": [5, 1, 3, 1] }),
    )
    .await;

    assert_eq!(response.status, StatusCode::CREATED);
    assert_eq!(response.body["weekdays"], json!([1, 3, 5]));
}

#[sqlx::test]
async fn create_trims_the_name(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = create_habit(&app, &token, json!({ "name": "  Read  " })).await;

    assert_eq!(response.body["name"], "Read");
}

#[sqlx::test]
async fn create_appends_new_habits_at_the_end(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    create_daily_habit(&app, &token, "First").await;
    create_daily_habit(&app, &token, "Second").await;

    let habits = list_habits(&app, &token, "").await;

    assert_eq!(names(&habits), ["First", "Second"]);
}

#[sqlx::test]
async fn create_rejects_blank_name(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = create_habit(&app, &token, json!({ "name": "   " })).await;

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(response.body["error"]["code"], "validation_failed");
    assert_eq!(
        response.body["error"]["details"]["fields"]["name"],
        json!(["blank"])
    );
}

#[sqlx::test]
async fn create_rejects_name_longer_than_100_characters(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = create_habit(&app, &token, json!({ "name": "a".repeat(101) })).await;

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        response.body["error"]["details"]["fields"]["name"],
        json!(["length"])
    );
}

#[sqlx::test]
async fn create_rejects_empty_weekdays(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = create_habit(&app, &token, json!({ "name": "Read", "weekdays": [] })).await;

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        response.body["error"]["details"]["fields"]["weekdays"],
        json!(["weekdays"])
    );
}

#[sqlx::test]
async fn create_rejects_weekday_outside_1_to_7(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = create_habit(&app, &token, json!({ "name": "Read", "weekdays": [0, 8] })).await;

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        response.body["error"]["details"]["fields"]["weekdays"],
        json!(["weekdays"])
    );
}

#[sqlx::test]
async fn create_defaults_to_hidden_on_low_energy_days(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = create_habit(&app, &token, json!({ "name": "Read" })).await;

    assert_eq!(response.body["low_energy"], false);
    assert_eq!(response.body["low_energy_name"], Value::Null);
}

#[sqlx::test]
async fn create_stores_the_low_energy_fields(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = create_habit(
        &app,
        &token,
        json!({ "name": "Run 5 km", "low_energy": true, "low_energy_name": "  Walk 5 minutes  " }),
    )
    .await;

    assert_eq!(response.status, StatusCode::CREATED);
    assert_eq!(response.body["low_energy"], true);
    assert_eq!(response.body["low_energy_name"], "Walk 5 minutes");
    let listed = list_habits(&app, &token, "").await;
    assert_eq!(listed.body[0]["low_energy_name"], "Walk 5 minutes");
}

#[sqlx::test]
async fn create_stores_a_blank_easier_name_as_null(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = create_habit(
        &app,
        &token,
        json!({ "name": "Read", "low_energy": true, "low_energy_name": "   " }),
    )
    .await;

    assert_eq!(response.body["low_energy"], true);
    assert_eq!(response.body["low_energy_name"], Value::Null);
}

#[sqlx::test]
async fn create_ignores_an_easier_name_when_the_habit_is_not_kept(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = create_habit(
        &app,
        &token,
        json!({ "name": "Read", "low_energy_name": "Read a page" }),
    )
    .await;

    assert_eq!(response.status, StatusCode::CREATED);
    assert_eq!(response.body["low_energy"], false);
    assert_eq!(response.body["low_energy_name"], Value::Null);
}

#[sqlx::test]
async fn create_rejects_easier_name_longer_than_100_characters(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = create_habit(
        &app,
        &token,
        json!({ "name": "Read", "low_energy": true, "low_energy_name": "a".repeat(101) }),
    )
    .await;

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        response.body["error"]["details"]["fields"]["low_energy_name"],
        json!(["length"])
    );
}

#[sqlx::test]
async fn create_requires_authentication(db: PgPool) {
    let app = common::app(db);

    let response = TestRequest::post("/api/v1/habits")
        .json(&json!({ "name": "Read" }))
        .send(&app)
        .await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
    assert_eq!(response.body["error"]["code"], "unauthorized");
}

// ---------------------------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn list_returns_only_the_habits_of_the_signed_in_user(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    create_daily_habit(&app, &ada, "Read").await;
    create_daily_habit(&app, &grace, "Run").await;

    let habits = list_habits(&app, &ada, "").await;

    assert_eq!(habits.status, StatusCode::OK);
    assert_eq!(names(&habits), ["Read"]);
}

#[sqlx::test]
async fn list_hides_archived_habits_unless_asked(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    create_daily_habit(&app, &token, "Read").await;
    let archived = create_daily_habit(&app, &token, "Old").await;
    patch_habit(&app, &token, &archived, json!({ "archived": true })).await;

    let active = list_habits(&app, &token, "").await;
    let all = list_habits(&app, &token, "?include_archived=true").await;

    assert_eq!(names(&active), ["Read"]);
    assert_eq!(names(&all), ["Read", "Old"]);
}

#[sqlx::test]
async fn list_rejects_malformed_query(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = list_habits(&app, &token, "?include_archived=maybe").await;

    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(response.body["error"]["code"], "invalid_request");
}

// ---------------------------------------------------------------------------------------------
// Update
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn update_changes_only_the_given_fields(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = create_daily_habit(&app, &token, "Read").await;

    let response = patch_habit(&app, &token, &id, json!({ "weekdays": [6, 7] })).await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["name"], "Read");
    assert_eq!(response.body["weekdays"], json!([6, 7]));
}

#[sqlx::test]
async fn update_turns_the_low_energy_flag_on_with_an_easier_name(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = create_daily_habit(&app, &token, "Run 5 km").await;

    let response = patch_habit(
        &app,
        &token,
        &id,
        json!({ "low_energy": true, "low_energy_name": "Walk 5 minutes" }),
    )
    .await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["low_energy"], true);
    assert_eq!(response.body["low_energy_name"], "Walk 5 minutes");
}

#[sqlx::test]
async fn update_without_low_energy_fields_keeps_them(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = create_habit(
        &app,
        &token,
        json!({ "name": "Run", "low_energy": true, "low_energy_name": "Walk" }),
    )
    .await
    .body["id"]
        .as_str()
        .unwrap()
        .to_owned();

    let response = patch_habit(&app, &token, &id, json!({ "name": "Run 5 km" })).await;

    assert_eq!(response.body["low_energy"], true);
    assert_eq!(response.body["low_energy_name"], "Walk");
}

#[sqlx::test]
async fn update_changes_only_the_easier_name_of_a_kept_habit(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = create_habit(
        &app,
        &token,
        json!({ "name": "Run", "low_energy": true, "low_energy_name": "Walk" }),
    )
    .await
    .body["id"]
        .as_str()
        .unwrap()
        .to_owned();

    let renamed = patch_habit(&app, &token, &id, json!({ "low_energy_name": "Stretch" })).await;
    let cleared = patch_habit(&app, &token, &id, json!({ "low_energy_name": "" })).await;

    assert_eq!(renamed.body["low_energy"], true);
    assert_eq!(renamed.body["low_energy_name"], "Stretch");
    assert_eq!(cleared.body["low_energy"], true);
    assert_eq!(cleared.body["low_energy_name"], Value::Null);
}

#[sqlx::test]
async fn update_turning_the_flag_off_clears_the_easier_name(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = create_habit(
        &app,
        &token,
        json!({ "name": "Run", "low_energy": true, "low_energy_name": "Walk" }),
    )
    .await
    .body["id"]
        .as_str()
        .unwrap()
        .to_owned();

    // The name in the same request is ignored: a habit that is not kept has no easier version.
    let response = patch_habit(
        &app,
        &token,
        &id,
        json!({ "low_energy": false, "low_energy_name": "Ignored" }),
    )
    .await;

    assert_eq!(response.body["low_energy"], false);
    assert_eq!(response.body["low_energy_name"], Value::Null);
    let listed = list_habits(&app, &token, "").await;
    assert_eq!(listed.body[0]["low_energy_name"], Value::Null);
}

#[sqlx::test]
async fn update_rejects_easier_name_longer_than_100_characters(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = create_daily_habit(&app, &token, "Read").await;

    let response = patch_habit(
        &app,
        &token,
        &id,
        json!({ "low_energy": true, "low_energy_name": "a".repeat(101) }),
    )
    .await;

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        response.body["error"]["details"]["fields"]["low_energy_name"],
        json!(["length"])
    );
}

#[sqlx::test]
async fn update_of_another_users_low_energy_fields_is_not_found(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let id = create_daily_habit(&app, &ada, "Read").await;

    let response = patch_habit(&app, &grace, &id, json!({ "low_energy": true })).await;

    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert_eq!(
        list_habits(&app, &ada, "").await.body[0]["low_energy"],
        false
    );
}

#[sqlx::test]
async fn update_archives_and_restores_a_habit(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = create_daily_habit(&app, &token, "Read").await;

    let archived = patch_habit(&app, &token, &id, json!({ "archived": true })).await;
    let restored = patch_habit(&app, &token, &id, json!({ "archived": false })).await;

    assert!(archived.body["archived_at"].is_string());
    assert_eq!(restored.body["archived_at"], Value::Null);
}

#[sqlx::test]
async fn update_reorders_habits_by_sort_order(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    create_daily_habit(&app, &token, "First").await;
    let second = create_daily_habit(&app, &token, "Second").await;

    patch_habit(&app, &token, &second, json!({ "sort_order": -1 })).await;

    let habits = list_habits(&app, &token, "").await;
    assert_eq!(names(&habits), ["Second", "First"]);
}

#[sqlx::test]
async fn update_rejects_invalid_fields(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = create_daily_habit(&app, &token, "Read").await;

    let response = patch_habit(&app, &token, &id, json!({ "name": "", "weekdays": [9] })).await;

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    let fields = &response.body["error"]["details"]["fields"];
    assert!(fields["name"].is_array());
    assert!(fields["weekdays"].is_array());
}

#[sqlx::test]
async fn update_of_another_users_habit_is_not_found(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let id = create_daily_habit(&app, &ada, "Read").await;

    let response = patch_habit(&app, &grace, &id, json!({ "name": "Hacked" })).await;

    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert_eq!(response.body["error"]["code"], "not_found");
    assert_eq!(names(&list_habits(&app, &ada, "").await), ["Read"]);
}

#[sqlx::test]
async fn update_rejects_malformed_id(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = patch_habit(&app, &token, "not-a-uuid", json!({ "name": "Read" })).await;

    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(response.body["error"]["code"], "invalid_request");
}

// ---------------------------------------------------------------------------------------------
// Delete
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn delete_removes_the_habit_and_its_history(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let id = create_daily_habit(&app, &token, "Read").await;
    complete(&app, &token, &id, today(&db).await).await;

    let response = TestRequest::new(Method::DELETE, &format!("/api/v1/habits/{id}"))
        .bearer(&token)
        .send(&app)
        .await;

    assert_eq!(response.status, StatusCode::NO_CONTENT);
    assert!(names(&list_habits(&app, &token, "?include_archived=true").await).is_empty());
    let completions: i64 = sqlx::query_scalar("SELECT count(*) FROM habit_completions")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(completions, 0);
}

#[sqlx::test]
async fn delete_of_another_users_habit_is_not_found(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let id = create_daily_habit(&app, &ada, "Read").await;

    let response = TestRequest::new(Method::DELETE, &format!("/api/v1/habits/{id}"))
        .bearer(&grace)
        .send(&app)
        .await;

    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert_eq!(names(&list_habits(&app, &ada, "").await), ["Read"]);
}

// ---------------------------------------------------------------------------------------------
// Completions
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn completing_twice_is_idempotent(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let id = create_daily_habit(&app, &token, "Read").await;
    let today = today(&db).await;

    let first = complete(&app, &token, &id, today).await;
    let second = complete(&app, &token, &id, today).await;

    assert_eq!(first.status, StatusCode::NO_CONTENT);
    assert_eq!(second.status, StatusCode::NO_CONTENT);
    assert_eq!(only_streak(&app, &token).await, 1);
}

#[sqlx::test]
async fn uncompleting_removes_the_completion_and_is_idempotent(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let id = create_daily_habit(&app, &token, "Read").await;
    let today = today(&db).await;
    complete(&app, &token, &id, today).await;

    let first = uncomplete(&app, &token, &id, today).await;
    let second = uncomplete(&app, &token, &id, today).await;

    assert_eq!(first.status, StatusCode::NO_CONTENT);
    assert_eq!(second.status, StatusCode::NO_CONTENT);
    assert_eq!(only_streak(&app, &token).await, 0);
}

#[sqlx::test]
async fn completing_a_future_date_is_rejected(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let id = create_daily_habit(&app, &token, "Read").await;
    let tomorrow = today(&db).await.succ_opt().unwrap();

    let response = complete(&app, &token, &id, tomorrow).await;

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        response.body["error"]["details"]["fields"]["date"],
        json!(["future"])
    );
}

#[sqlx::test]
async fn completing_a_day_the_habit_is_not_scheduled_for_is_rejected(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let today = today(&db).await;
    let yesterday = days_ago(today, 1);
    // Scheduled only for today's weekday, so yesterday is never a scheduled day.
    let habit = create_habit(
        &app,
        &token,
        json!({ "name": "Weekly", "weekdays": [weekday(today)] }),
    )
    .await;
    let id = habit.body["id"].as_str().unwrap();

    let response = complete(&app, &token, id, yesterday).await;

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        response.body["error"]["details"]["fields"]["date"],
        json!(["not_scheduled"])
    );
}

#[sqlx::test]
async fn completing_rejects_malformed_date(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = create_daily_habit(&app, &token, "Read").await;

    let response = TestRequest::new(
        Method::PUT,
        &format!("/api/v1/habits/{id}/completions/2026-13-45"),
    )
    .bearer(&token)
    .send(&app)
    .await;

    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(response.body["error"]["code"], "invalid_request");
}

#[sqlx::test]
async fn completing_another_users_habit_is_not_found(db: PgPool) {
    let app = common::app(db.clone());
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let id = create_daily_habit(&app, &ada, "Read").await;
    let today = today(&db).await;

    let completed = complete(&app, &grace, &id, today).await;
    let uncompleted = uncomplete(&app, &grace, &id, today).await;

    assert_eq!(completed.status, StatusCode::NOT_FOUND);
    assert_eq!(uncompleted.status, StatusCode::NOT_FOUND);
    assert_eq!(only_streak(&app, &ada).await, 0);
}

// ---------------------------------------------------------------------------------------------
// Streaks
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn streak_counts_consecutive_days_ending_today(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let id = create_daily_habit(&app, &token, "Read").await;
    let today = today(&db).await;
    for days in 0..3 {
        complete(&app, &token, &id, days_ago(today, days)).await;
    }

    assert_eq!(only_streak(&app, &token).await, 3);
}

#[sqlx::test]
async fn streak_is_not_broken_by_today_being_unchecked(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let id = create_daily_habit(&app, &token, "Read").await;
    let today = today(&db).await;
    complete(&app, &token, &id, days_ago(today, 1)).await;
    complete(&app, &token, &id, days_ago(today, 2)).await;

    assert_eq!(only_streak(&app, &token).await, 2);
}

#[sqlx::test]
async fn streak_is_broken_by_a_missed_day(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let id = create_daily_habit(&app, &token, "Read").await;
    let today = today(&db).await;
    complete(&app, &token, &id, today).await;
    // Yesterday is missed.
    complete(&app, &token, &id, days_ago(today, 2)).await;
    complete(&app, &token, &id, days_ago(today, 3)).await;

    assert_eq!(only_streak(&app, &token).await, 1);
}

#[sqlx::test]
async fn streak_is_zero_when_yesterday_was_missed_and_today_is_unchecked(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let id = create_daily_habit(&app, &token, "Read").await;
    let today = today(&db).await;
    complete(&app, &token, &id, days_ago(today, 2)).await;

    assert_eq!(only_streak(&app, &token).await, 0);
}

#[sqlx::test]
async fn streak_skips_days_the_habit_is_not_scheduled_for(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let today = today(&db).await;
    // Scheduled once a week, on today's weekday.
    let habit = create_habit(
        &app,
        &token,
        json!({ "name": "Weekly", "weekdays": [weekday(today)] }),
    )
    .await;
    let id = habit.body["id"].as_str().unwrap();
    complete(&app, &token, id, today).await;
    complete(&app, &token, id, days_ago(today, 7)).await;
    complete(&app, &token, id, days_ago(today, 14)).await;

    assert_eq!(only_streak(&app, &token).await, 3);
}

#[sqlx::test]
async fn streak_uses_the_current_schedule_for_the_past(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let id = create_daily_habit(&app, &token, "Read").await;
    let today = today(&db).await;
    complete(&app, &token, &id, today).await;
    complete(&app, &token, &id, days_ago(today, 1)).await;
    complete(&app, &token, &id, days_ago(today, 7)).await;

    // Narrowing the schedule to today's weekday makes the six days in between unscheduled, so
    // the two same-weekday completions become consecutive and yesterday's stops counting.
    patch_habit(&app, &token, &id, json!({ "weekdays": [weekday(today)] })).await;

    assert_eq!(only_streak(&app, &token).await, 2);
}

// ---------------------------------------------------------------------------------------------
// Low-energy days
// ---------------------------------------------------------------------------------------------

/// Marks a date as a low-energy day for the user. The API only changes today, so tests write
/// past days directly.
async fn mark_low_energy_day(db: &PgPool, email: &str, date: NaiveDate) {
    sqlx::query(
        "INSERT INTO low_energy_days (user_id, date) SELECT id, $2 FROM users WHERE email = $1",
    )
    .bind(email)
    .bind(date)
    .execute(db)
    .await
    .unwrap();
}

#[sqlx::test]
async fn streak_continues_across_a_low_energy_day_for_a_hidden_habit(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let id = create_daily_habit(&app, &token, "Read").await;
    let today = today(&db).await;
    mark_low_energy_day(&db, ADA, days_ago(today, 1)).await;
    for days in [0, 2, 3] {
        complete(&app, &token, &id, days_ago(today, days)).await;
    }

    assert_eq!(only_streak(&app, &token).await, 3);
}

#[sqlx::test]
async fn streak_of_a_kept_habit_is_broken_by_missing_a_low_energy_day(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let response = create_habit(&app, &token, json!({ "name": "Read", "low_energy": true })).await;
    let id = response.body["id"].as_str().unwrap();
    let today = today(&db).await;
    mark_low_energy_day(&db, ADA, days_ago(today, 1)).await;
    for days in [0, 2, 3] {
        complete(&app, &token, id, days_ago(today, days)).await;
    }

    assert_eq!(only_streak(&app, &token).await, 1);
}

#[sqlx::test]
async fn streak_of_a_kept_habit_grows_when_done_on_a_low_energy_day(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let response = create_habit(&app, &token, json!({ "name": "Read", "low_energy": true })).await;
    let id = response.body["id"].as_str().unwrap();
    let today = today(&db).await;
    mark_low_energy_day(&db, ADA, days_ago(today, 1)).await;
    for days in 0..4 {
        let completed = complete(&app, &token, id, days_ago(today, days)).await;
        assert_eq!(completed.status, StatusCode::NO_CONTENT);
    }

    assert_eq!(only_streak(&app, &token).await, 4);
}

#[sqlx::test]
async fn streak_changes_when_the_habit_stops_being_kept(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let response = create_habit(&app, &token, json!({ "name": "Read", "low_energy": true })).await;
    let id = response.body["id"].as_str().unwrap();
    let today = today(&db).await;
    mark_low_energy_day(&db, ADA, days_ago(today, 1)).await;
    for days in [0, 2, 3] {
        complete(&app, &token, id, days_ago(today, days)).await;
    }

    // The current settings apply to the past: once hidden, the low-energy day no longer breaks it.
    patch_habit(&app, &token, id, json!({ "low_energy": false })).await;

    assert_eq!(only_streak(&app, &token).await, 3);
}

#[sqlx::test]
async fn completing_a_hidden_habit_on_a_low_energy_day_is_rejected(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let id = create_daily_habit(&app, &token, "Read").await;
    let today = today(&db).await;
    let yesterday = days_ago(today, 1);
    mark_low_energy_day(&db, ADA, yesterday).await;

    let response = complete(&app, &token, &id, yesterday).await;

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        response.body["error"]["details"]["fields"]["date"],
        json!(["not_scheduled"])
    );
}

#[sqlx::test]
async fn completing_a_hidden_habit_today_after_switching_to_low_energy_is_rejected(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let id = create_daily_habit(&app, &token, "Read").await;
    let today = today(&db).await;
    let switched = TestRequest::new(Method::PUT, &format!("/api/v1/days/{today}/mode"))
        .bearer(&token)
        .json(&json!({ "mode": "low_energy" }))
        .send(&app)
        .await;
    assert_eq!(switched.status, StatusCode::NO_CONTENT);

    let response = complete(&app, &token, &id, today).await;

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        response.body["error"]["details"]["fields"]["date"],
        json!(["not_scheduled"])
    );
}

#[sqlx::test]
async fn a_low_energy_day_of_another_user_does_not_affect_the_habit(db: PgPool) {
    let app = common::app(db.clone());
    let ada = sign_up(&app, ADA).await;
    sign_up(&app, GRACE).await;
    let id = create_daily_habit(&app, &ada, "Read").await;
    let today = today(&db).await;
    mark_low_energy_day(&db, GRACE, today).await;

    let response = complete(&app, &ada, &id, today).await;

    assert_eq!(response.status, StatusCode::NO_CONTENT);
}
