//! Integration tests for the weekly statistics endpoint.

// Test helpers are not `#[test]` functions, so the clippy.toml test exemption does not reach them.
#![allow(clippy::unwrap_used)]

use axum::{Router, http::Method};
use chrono::{Datelike, Days, NaiveDate};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::{TestRequest, TestResponse, sign_up, today};

mod common;

const ADA: &str = "ada@example.com";
const GRACE: &str = "grace@example.com";

/// The Monday of the week that contains `date`.
fn monday_of(date: NaiveDate) -> NaiveDate {
    date - Days::new(u64::from(date.weekday().num_days_from_monday()))
}

/// A Monday two weeks before the current week, so every day of it and of the week before it is
/// in the past and may be written freely.
async fn past_monday(db: &PgPool) -> NaiveDate {
    monday_of(today(db).await) - Days::new(14)
}

fn plus(date: NaiveDate, days: u64) -> NaiveDate {
    date + Days::new(days)
}

async fn get_week(app: &Router, token: &str, week_start: NaiveDate) -> TestResponse {
    TestRequest::get(&format!("/api/v1/stats/weekly?week_start={week_start}"))
        .bearer(token)
        .send(app)
        .await
}

async fn add_habit(app: &Router, token: &str, body: Value) -> String {
    let response = TestRequest::post("/api/v1/habits")
        .bearer(token)
        .json(&body)
        .send(app)
        .await;
    assert_eq!(response.status, 201);
    response.body["id"].as_str().unwrap().to_owned()
}

async fn complete(app: &Router, token: &str, habit_id: &str, date: NaiveDate) {
    let response = TestRequest::new(
        Method::PUT,
        &format!("/api/v1/habits/{habit_id}/completions/{date}"),
    )
    .bearer(token)
    .send(app)
    .await;
    assert_eq!(response.status, 204);
}

/// Creates a to-do for `date` and returns its ID.
async fn add_task(app: &Router, token: &str, date: NaiveDate, title: &str) -> String {
    let response = TestRequest::post("/api/v1/tasks")
        .bearer(token)
        .json(&json!({ "date": date, "title": title }))
        .send(app)
        .await;
    assert_eq!(response.status, 201);
    response.body["id"].as_str().unwrap().to_owned()
}

async fn finish_task(app: &Router, token: &str, id: &str) {
    let response = TestRequest::new(Method::PATCH, &format!("/api/v1/tasks/{id}"))
        .bearer(token)
        .json(&json!({ "is_done": true }))
        .send(app)
        .await;
    assert_eq!(response.status, 200);
}

async fn add_subject(app: &Router, token: &str, name: &str, color: &str) -> String {
    let response = TestRequest::post("/api/v1/subjects")
        .bearer(token)
        .json(&json!({ "name": name, "color": color }))
        .send(app)
        .await;
    assert_eq!(response.status, 201);
    response.body["id"].as_str().unwrap().to_owned()
}

/// Writes a study session directly: the API stamps sessions with the current time, so past
/// weeks can only be filled through the database.
async fn add_session(
    db: &PgPool,
    email: &str,
    subject_id: Option<&str>,
    status: &str,
    date: NaiveDate,
    seconds: i32,
) {
    let subject_id = subject_id.map(|id| id.parse::<uuid::Uuid>().unwrap());
    sqlx::query(
        "INSERT INTO study_sessions
             (user_id, subject_id, status, planned_minutes, started_at, ended_at,
              duration_seconds, local_date)
         SELECT id, $2, $3, 25, now(), now(), $4, $5 FROM users WHERE email = $1",
    )
    .bind(email)
    .bind(subject_id)
    .bind(status)
    .bind(seconds)
    .bind(date)
    .execute(db)
    .await
    .unwrap();
}

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

fn day(response: &TestResponse, index: usize) -> &Value {
    &response.body["days"][index]
}

#[sqlx::test]
async fn an_empty_week_lists_seven_days_from_monday_with_zeros(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let monday = past_monday(&db).await;

    let response = get_week(&app, &token, monday).await;

    assert_eq!(response.status, 200);
    assert_eq!(response.body["week_start"], json!(monday));
    assert_eq!(response.body["week_end"], json!(plus(monday, 6)));
    let days = response.body["days"].as_array().unwrap();
    assert_eq!(days.len(), 7);
    for (offset, entry) in days.iter().enumerate() {
        assert_eq!(entry["date"], json!(plus(monday, offset as u64)));
        assert_eq!(entry["mode"], "normal");
        assert_eq!(entry["study_seconds"], 0);
        assert_eq!(entry["pomodoros"], 0);
        assert_eq!(entry["habits_scheduled"], 0);
        assert_eq!(entry["habits_done"], 0);
        assert_eq!(entry["tasks_total"], 0);
        assert_eq!(entry["tasks_done"], 0);
        assert_eq!(entry["completion_rate"], Value::Null);
    }
    assert_eq!(response.body["subjects"], json!([]));
    assert_eq!(response.body["habits"], json!([]));
    assert_eq!(response.body["totals"]["study_seconds"], 0);
    assert_eq!(response.body["totals"]["pomodoros"], 0);
    assert_eq!(response.body["totals"]["completion_rate"], Value::Null);
    assert_eq!(response.body["previous_totals"]["study_seconds"], 0);
}

#[sqlx::test]
async fn the_week_defaults_to_the_current_week(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;

    let response = TestRequest::get("/api/v1/stats/weekly")
        .bearer(&token)
        .send(&app)
        .await;

    assert_eq!(response.status, 200);
    assert_eq!(
        response.body["week_start"],
        json!(monday_of(today(&db).await))
    );
}

#[sqlx::test]
async fn a_week_start_that_is_not_a_monday_is_a_validation_error(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let tuesday = plus(past_monday(&db).await, 1);

    let response = get_week(&app, &token, tuesday).await;

    assert_eq!(response.status, 422);
    assert_eq!(response.body["error"]["code"], "validation_failed");
    assert_eq!(
        response.body["error"]["details"]["fields"]["week_start"],
        json!(["not_monday"])
    );
}

#[sqlx::test]
async fn a_malformed_week_start_is_a_bad_request(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = TestRequest::get("/api/v1/stats/weekly?week_start=soon")
        .bearer(&token)
        .send(&app)
        .await;

    assert_eq!(response.status, 400);
    assert_eq!(response.body["error"]["code"], "invalid_request");
}

#[sqlx::test]
async fn stats_require_authentication(db: PgPool) {
    let app = common::app(db);

    let response = TestRequest::get("/api/v1/stats/weekly").send(&app).await;

    assert_eq!(response.status, 401);
    assert_eq!(response.body["error"]["code"], "unauthorized");
}

#[sqlx::test]
async fn study_time_is_summed_per_day_and_per_subject(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let monday = past_monday(&db).await;
    let math = add_subject(&app, &token, "Math", "#3B82F6").await;
    let art = add_subject(&app, &token, "Art", "#EF4444").await;
    add_session(&db, ADA, Some(&math), "completed", monday, 1500).await;
    add_session(&db, ADA, Some(&math), "completed", plus(monday, 2), 1200).await;
    add_session(&db, ADA, Some(&art), "completed", plus(monday, 2), 600).await;
    add_session(&db, ADA, None, "completed", plus(monday, 2), 300).await;

    let response = get_week(&app, &token, monday).await;

    assert_eq!(day(&response, 0)["study_seconds"], 1500);
    assert_eq!(day(&response, 0)["pomodoros"], 1);
    assert_eq!(day(&response, 1)["study_seconds"], 0);
    assert_eq!(day(&response, 2)["study_seconds"], 2100);
    assert_eq!(day(&response, 2)["pomodoros"], 3);
    assert_eq!(response.body["totals"]["study_seconds"], 3600);
    assert_eq!(response.body["totals"]["pomodoros"], 4);

    // Longest study time first; sessions without a subject form one entry with null fields.
    let subjects = response.body["subjects"].as_array().unwrap();
    assert_eq!(subjects.len(), 3);
    assert_eq!(subjects[0]["subject_id"], json!(math));
    assert_eq!(subjects[0]["name"], "Math");
    assert_eq!(subjects[0]["color"], "#3B82F6");
    assert_eq!(subjects[0]["study_seconds"], 2700);
    assert_eq!(subjects[0]["pomodoros"], 2);
    assert_eq!(subjects[1]["name"], "Art");
    assert_eq!(subjects[1]["study_seconds"], 600);
    assert_eq!(subjects[2]["subject_id"], Value::Null);
    assert_eq!(subjects[2]["name"], Value::Null);
    assert_eq!(subjects[2]["color"], Value::Null);
    assert_eq!(subjects[2]["study_seconds"], 300);
}

#[sqlx::test]
async fn cancelled_and_running_sessions_do_not_count(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let monday = past_monday(&db).await;
    add_session(&db, ADA, None, "completed", monday, 900).await;
    add_session(&db, ADA, None, "cancelled", monday, 800).await;
    sqlx::query(
        "INSERT INTO study_sessions (user_id, status, planned_minutes, started_at, local_date)
         SELECT id, 'running', 25, now(), $2 FROM users WHERE email = $1",
    )
    .bind(ADA)
    .bind(monday)
    .execute(&db)
    .await
    .unwrap();

    let response = get_week(&app, &token, monday).await;

    assert_eq!(day(&response, 0)["study_seconds"], 900);
    assert_eq!(day(&response, 0)["pomodoros"], 1);
    assert_eq!(response.body["subjects"].as_array().unwrap().len(), 1);
}

#[sqlx::test]
async fn sessions_outside_the_week_are_left_out_of_it(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let monday = past_monday(&db).await;
    add_session(&db, ADA, None, "completed", monday - Days::new(1), 111).await;
    add_session(&db, ADA, None, "completed", plus(monday, 7), 222).await;

    let response = get_week(&app, &token, monday).await;

    assert_eq!(response.body["totals"]["study_seconds"], 0);
    assert_eq!(response.body["subjects"], json!([]));
}

#[sqlx::test]
async fn the_checklist_rate_counts_habits_and_to_dos(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let monday = past_monday(&db).await;
    let read = add_habit(&app, &token, json!({ "name": "Read" })).await;
    add_habit(&app, &token, json!({ "name": "Stretch" })).await;
    complete(&app, &token, &read, monday).await;
    let done = add_task(&app, &token, monday, "Pay rent").await;
    add_task(&app, &token, monday, "Call mum").await;
    finish_task(&app, &token, &done).await;

    let response = get_week(&app, &token, monday).await;

    // Two habits scheduled, one done; two to-dos, one done: 2 of 4.
    let monday_stats = day(&response, 0);
    assert_eq!(monday_stats["habits_scheduled"], 2);
    assert_eq!(monday_stats["habits_done"], 1);
    assert_eq!(monday_stats["tasks_total"], 2);
    assert_eq!(monday_stats["tasks_done"], 1);
    assert_eq!(monday_stats["completion_rate"], 0.5);
    // The other days have two habits scheduled and nothing done.
    assert_eq!(day(&response, 1)["habits_scheduled"], 2);
    assert_eq!(day(&response, 1)["completion_rate"], 0.0);
    // The week's rate pools every item: 2 done of 4 + 6 * 2 scheduled.
    let rate = response.body["totals"]["completion_rate"].as_f64().unwrap();
    assert!((rate - 2.0 / 16.0).abs() < 1e-9, "{rate}");
}

#[sqlx::test]
async fn a_habit_only_counts_on_its_weekdays(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let monday = past_monday(&db).await;
    add_habit(&app, &token, json!({ "name": "Gym", "weekdays": [1, 3] })).await;

    let response = get_week(&app, &token, monday).await;

    let scheduled: Vec<i64> = response.body["days"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["habits_scheduled"].as_i64().unwrap())
        .collect();
    assert_eq!(scheduled, [1, 0, 1, 0, 0, 0, 0]);
}

#[sqlx::test]
async fn archived_habits_are_left_out(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let monday = past_monday(&db).await;
    let id = add_habit(&app, &token, json!({ "name": "Old" })).await;
    complete(&app, &token, &id, monday).await;
    let archived = TestRequest::new(Method::PATCH, &format!("/api/v1/habits/{id}"))
        .bearer(&token)
        .json(&json!({ "archived": true }))
        .send(&app)
        .await;
    assert_eq!(archived.status, 200);

    let response = get_week(&app, &token, monday).await;

    assert_eq!(day(&response, 0)["habits_scheduled"], 0);
    assert_eq!(day(&response, 0)["habits_done"], 0);
    assert_eq!(response.body["habits"], json!([]));
}

#[sqlx::test]
async fn low_energy_days_are_marked_and_hide_habits_that_are_not_kept(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let monday = past_monday(&db).await;
    let tuesday = plus(monday, 1);
    add_habit(&app, &token, json!({ "name": "Run" })).await;
    add_habit(
        &app,
        &token,
        json!({ "name": "Read", "low_energy": true, "low_energy_name": "Read a page" }),
    )
    .await;
    mark_low_energy_day(&db, ADA, tuesday).await;

    let response = get_week(&app, &token, monday).await;

    assert_eq!(day(&response, 0)["mode"], "normal");
    assert_eq!(day(&response, 0)["habits_scheduled"], 2);
    assert_eq!(day(&response, 1)["mode"], "low_energy");
    assert_eq!(day(&response, 1)["habits_scheduled"], 1);
}

#[sqlx::test]
async fn a_habit_done_on_a_day_it_is_hidden_is_not_counted(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let monday = past_monday(&db).await;
    let run = add_habit(&app, &token, json!({ "name": "Run" })).await;
    // Done while the day was normal; the day was marked low-energy afterwards.
    complete(&app, &token, &run, monday).await;
    mark_low_energy_day(&db, ADA, monday).await;

    let response = get_week(&app, &token, monday).await;

    assert_eq!(day(&response, 0)["habits_scheduled"], 0);
    assert_eq!(day(&response, 0)["habits_done"], 0);
    assert_eq!(day(&response, 0)["completion_rate"], Value::Null);
}

#[sqlx::test]
async fn to_dos_count_on_a_low_energy_day(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let monday = past_monday(&db).await;
    mark_low_energy_day(&db, ADA, monday).await;
    let id = add_task(&app, &token, monday, "Laundry").await;
    finish_task(&app, &token, &id).await;

    let response = get_week(&app, &token, monday).await;

    assert_eq!(day(&response, 0)["tasks_total"], 1);
    assert_eq!(day(&response, 0)["completion_rate"], 1.0);
}

#[sqlx::test]
async fn streaks_list_the_current_and_the_longest_run(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let today = today(&db).await;
    let monday = past_monday(&db).await;
    let id = add_habit(&app, &token, json!({ "name": "Read" })).await;
    // A run of four days long ago, a gap, then a run of two ending today.
    for offset in [20, 19, 18, 17, 1, 0] {
        complete(&app, &token, &id, today - Days::new(offset)).await;
    }

    let response = get_week(&app, &token, monday).await;

    let habits = response.body["habits"].as_array().unwrap();
    assert_eq!(habits.len(), 1);
    assert_eq!(habits[0]["id"], json!(id));
    assert_eq!(habits[0]["name"], "Read");
    assert_eq!(habits[0]["current_streak"], 2);
    assert_eq!(habits[0]["longest_streak"], 4);
}

#[sqlx::test]
async fn the_longest_streak_skips_days_a_habit_is_hidden(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let today = today(&db).await;
    let monday = past_monday(&db).await;
    let id = add_habit(&app, &token, json!({ "name": "Run" })).await;
    // Done on three days with a low-energy day (hidden for this habit) in between.
    for offset in [5, 3, 2] {
        complete(&app, &token, &id, today - Days::new(offset)).await;
    }
    mark_low_energy_day(&db, ADA, today - Days::new(4)).await;

    let response = get_week(&app, &token, monday).await;

    assert_eq!(response.body["habits"][0]["longest_streak"], 3);
}

#[sqlx::test]
async fn a_habit_that_was_never_done_has_streaks_of_zero(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let monday = past_monday(&db).await;
    add_habit(&app, &token, json!({ "name": "Read" })).await;

    let response = get_week(&app, &token, monday).await;

    assert_eq!(response.body["habits"][0]["current_streak"], 0);
    assert_eq!(response.body["habits"][0]["longest_streak"], 0);
}

#[sqlx::test]
async fn the_previous_week_is_summarized_for_comparison(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let monday = past_monday(&db).await;
    let previous_monday = monday - Days::new(7);
    add_session(&db, ADA, None, "completed", previous_monday, 1000).await;
    add_session(&db, ADA, None, "completed", plus(previous_monday, 6), 500).await;
    add_session(&db, ADA, None, "completed", monday, 2000).await;
    let task = add_task(&app, &token, plus(previous_monday, 3), "Tidy").await;
    finish_task(&app, &token, &task).await;

    let response = get_week(&app, &token, monday).await;

    assert_eq!(response.body["totals"]["study_seconds"], 2000);
    assert_eq!(response.body["totals"]["pomodoros"], 1);
    assert_eq!(response.body["previous_totals"]["study_seconds"], 1500);
    assert_eq!(response.body["previous_totals"]["pomodoros"], 2);
    assert_eq!(response.body["previous_totals"]["completion_rate"], 1.0);
    assert_eq!(response.body["totals"]["completion_rate"], Value::Null);
}

#[sqlx::test]
async fn another_users_data_is_not_included(db: PgPool) {
    let app = common::app(db.clone());
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let monday = past_monday(&db).await;
    add_session(&db, GRACE, None, "completed", monday, 3000).await;
    add_habit(&app, &grace, json!({ "name": "Grace's habit" })).await;
    add_task(&app, &grace, monday, "Grace's task").await;
    mark_low_energy_day(&db, GRACE, monday).await;

    let response = get_week(&app, &ada, monday).await;

    assert_eq!(response.body["totals"]["study_seconds"], 0);
    assert_eq!(response.body["subjects"], json!([]));
    assert_eq!(response.body["habits"], json!([]));
    assert_eq!(day(&response, 0)["mode"], "normal");
    assert_eq!(day(&response, 0)["tasks_total"], 0);
    assert_eq!(day(&response, 0)["habits_scheduled"], 0);
}
