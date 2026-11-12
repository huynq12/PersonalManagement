//! Integration tests for the study session endpoints.

// Test helpers are not `#[test]` functions, so the clippy.toml test exemption does not reach them.
#![allow(clippy::unwrap_used)]

use axum::{
    Router,
    http::{Method, StatusCode},
};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::{TestRequest, TestResponse, sign_up, today};

mod common;

const ADA: &str = "ada@example.com";
const GRACE: &str = "grace@example.com";
const BLUE: &str = "#3B82F6";

async fn start(app: &Router, token: &str, body: Value) -> TestResponse {
    TestRequest::post("/api/v1/study-sessions")
        .bearer(token)
        .json(&body)
        .send(app)
        .await
}

/// Starts a session with the default length and returns its ID.
async fn start_default(app: &Router, token: &str) -> String {
    let response = start(app, token, json!({})).await;
    assert_eq!(response.status, StatusCode::CREATED);
    response.body["id"].as_str().unwrap().to_owned()
}

async fn current(app: &Router, token: &str) -> TestResponse {
    TestRequest::get("/api/v1/study-sessions/current")
        .bearer(token)
        .send(app)
        .await
}

async fn finish(app: &Router, token: &str, id: &str, action: &str) -> TestResponse {
    TestRequest::post(&format!("/api/v1/study-sessions/{id}/{action}"))
        .bearer(token)
        .send(app)
        .await
}

async fn history(app: &Router, token: &str, query: &str) -> TestResponse {
    TestRequest::get(&format!("/api/v1/study-sessions{query}"))
        .bearer(token)
        .send(app)
        .await
}

async fn delete_session(app: &Router, token: &str, id: &str) -> TestResponse {
    TestRequest::new(Method::DELETE, &format!("/api/v1/study-sessions/{id}"))
        .bearer(token)
        .send(app)
        .await
}

async fn create_subject(app: &Router, token: &str, name: &str) -> String {
    let response = TestRequest::post("/api/v1/subjects")
        .bearer(token)
        .json(&json!({ "name": name, "color": BLUE }))
        .send(app)
        .await;
    assert_eq!(response.status, StatusCode::CREATED);
    response.body["id"].as_str().unwrap().to_owned()
}

/// Pretends the session started `minutes` ago, so the tests need not wait.
async fn started_minutes_ago(db: &PgPool, id: &str, minutes: i32) {
    sqlx::query(
        "UPDATE study_sessions SET started_at = now() - make_interval(mins => $2) WHERE id = $1::uuid",
    )
    .bind(id)
    .bind(minutes)
    .execute(db)
    .await
    .unwrap();
}

fn error_code(response: &TestResponse) -> &str {
    response.body["error"]["code"].as_str().unwrap()
}

fn ids(response: &TestResponse) -> Vec<&str> {
    response
        .body
        .as_array()
        .unwrap()
        .iter()
        .map(|session| session["id"].as_str().unwrap())
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Start
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn start_returns_a_running_session_with_the_default_length(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;

    let response = start(&app, &token, json!({})).await;

    assert_eq!(response.status, StatusCode::CREATED);
    let session = &response.body;
    assert_eq!(session["status"], "running");
    assert_eq!(session["planned_minutes"], 25);
    assert_eq!(session["subject_id"], Value::Null);
    assert_eq!(session["ended_at"], Value::Null);
    assert_eq!(session["duration_seconds"], Value::Null);
    assert_eq!(session["local_date"], today(&db).await.to_string());
    assert!(session["started_at"].is_string());
}

#[sqlx::test]
async fn start_takes_the_length_from_the_users_settings(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    sqlx::query("UPDATE user_settings SET focus_minutes = 50")
        .execute(&db)
        .await
        .unwrap();

    let response = start(&app, &token, json!({})).await;

    assert_eq!(response.body["planned_minutes"], 50);
}

#[sqlx::test]
async fn start_accepts_a_subject_and_a_length(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let subject_id = create_subject(&app, &token, "Math").await;

    let response = start(
        &app,
        &token,
        json!({ "subject_id": subject_id, "planned_minutes": 40 }),
    )
    .await;

    assert_eq!(response.status, StatusCode::CREATED);
    assert_eq!(response.body["subject_id"], subject_id);
    assert_eq!(response.body["planned_minutes"], 40);
}

#[sqlx::test]
async fn start_rejects_a_length_out_of_range(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    for minutes in [0, 181, -5] {
        let response = start(&app, &token, json!({ "planned_minutes": minutes })).await;

        assert_eq!(
            response.status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{minutes}"
        );
        assert_eq!(error_code(&response), "validation_failed");
        assert!(
            response.body["error"]["details"]["fields"]["planned_minutes"].is_array(),
            "{minutes}"
        );
    }
}

#[sqlx::test]
async fn start_rejects_a_subject_that_does_not_exist(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = start(
        &app,
        &token,
        json!({ "subject_id": "00000000-0000-4000-8000-000000000000" }),
    )
    .await;

    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&response), "not_found");
}

#[sqlx::test]
async fn start_rejects_a_subject_of_another_user(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let subject_id = create_subject(&app, &grace, "Math").await;

    let response = start(&app, &ada, json!({ "subject_id": subject_id })).await;

    assert_eq!(response.status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn start_rejects_an_archived_subject(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let subject_id = create_subject(&app, &token, "Math").await;
    let archived = TestRequest::new(Method::PATCH, &format!("/api/v1/subjects/{subject_id}"))
        .bearer(&token)
        .json(&json!({ "archived": true }))
        .send(&app)
        .await;
    assert_eq!(archived.status, StatusCode::OK);

    let response = start(&app, &token, json!({ "subject_id": subject_id })).await;

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        response.body["error"]["details"]["fields"]["subject_id"],
        json!(["archived"])
    );
}

#[sqlx::test]
async fn start_fails_while_a_session_is_running(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    start_default(&app, &token).await;

    let response = start(&app, &token, json!({})).await;

    assert_eq!(response.status, StatusCode::CONFLICT);
    assert_eq!(error_code(&response), "study_session_already_running");
}

#[sqlx::test]
async fn start_is_allowed_again_once_the_session_is_cancelled(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = start_default(&app, &token).await;
    finish(&app, &token, &id, "cancel").await;

    let response = start(&app, &token, json!({})).await;

    assert_eq!(response.status, StatusCode::CREATED);
}

#[sqlx::test]
async fn two_users_can_run_sessions_at_the_same_time(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    start_default(&app, &ada).await;

    let response = start(&app, &grace, json!({})).await;

    assert_eq!(response.status, StatusCode::CREATED);
}

#[sqlx::test]
async fn start_requires_authentication(db: PgPool) {
    let app = common::app(db);

    let response = TestRequest::post("/api/v1/study-sessions")
        .json(&json!({}))
        .send(&app)
        .await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------------------------
// Current
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn current_answers_no_content_without_a_running_session(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = current(&app, &token).await;

    assert_eq!(response.status, StatusCode::NO_CONTENT);
}

#[sqlx::test]
async fn current_returns_the_running_session(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = start_default(&app, &token).await;

    let response = current(&app, &token).await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["id"], id);
    assert_eq!(response.body["status"], "running");
}

#[sqlx::test]
async fn current_ignores_finished_sessions_and_other_users(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let id = start_default(&app, &ada).await;
    start_default(&app, &grace).await;
    finish(&app, &ada, &id, "complete").await;

    let response = current(&app, &ada).await;

    assert_eq!(response.status, StatusCode::NO_CONTENT);
}

// ---------------------------------------------------------------------------------------------
// Complete
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn complete_records_the_elapsed_time(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let id = start_default(&app, &token).await;
    started_minutes_ago(&db, &id, 10).await;

    let response = finish(&app, &token, &id, "complete").await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["status"], "completed");
    assert!(response.body["ended_at"].is_string());
    let seconds = response.body["duration_seconds"].as_i64().unwrap();
    assert!((600..620).contains(&seconds), "{seconds}");
}

#[sqlx::test]
async fn complete_caps_the_time_at_the_planned_length(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let id = start_default(&app, &token).await;
    started_minutes_ago(&db, &id, 90).await;

    let response = finish(&app, &token, &id, "complete").await;

    assert_eq!(response.body["duration_seconds"], 25 * 60);
}

#[sqlx::test]
async fn complete_cannot_be_repeated(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = start_default(&app, &token).await;
    finish(&app, &token, &id, "complete").await;

    for action in ["complete", "cancel"] {
        let response = finish(&app, &token, &id, action).await;

        assert_eq!(response.status, StatusCode::CONFLICT, "{action}");
        assert_eq!(
            error_code(&response),
            "study_session_not_running",
            "{action}"
        );
    }
}

#[sqlx::test]
async fn complete_hides_the_session_of_another_user(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let id = start_default(&app, &ada).await;

    let response = finish(&app, &grace, &id, "complete").await;

    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert_eq!(current(&app, &ada).await.body["status"], "running");
}

#[sqlx::test]
async fn complete_answers_not_found_for_an_unknown_session(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = finish(
        &app,
        &token,
        "00000000-0000-4000-8000-000000000000",
        "complete",
    )
    .await;

    assert_eq!(response.status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn complete_rejects_a_malformed_id(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = finish(&app, &token, "not-a-uuid", "complete").await;

    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&response), "invalid_request");
}

// ---------------------------------------------------------------------------------------------
// Cancel
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn cancel_ends_the_session_without_a_duration(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let id = start_default(&app, &token).await;
    started_minutes_ago(&db, &id, 10).await;

    let response = finish(&app, &token, &id, "cancel").await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["status"], "cancelled");
    assert!(response.body["ended_at"].is_string());
    assert_eq!(response.body["duration_seconds"], Value::Null);
    assert_eq!(current(&app, &token).await.status, StatusCode::NO_CONTENT);
}

#[sqlx::test]
async fn cancel_cannot_undo_a_completed_session(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = start_default(&app, &token).await;
    finish(&app, &token, &id, "complete").await;

    let response = finish(&app, &token, &id, "cancel").await;

    assert_eq!(response.status, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn cancel_hides_the_session_of_another_user(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let id = start_default(&app, &ada).await;

    let response = finish(&app, &grace, &id, "cancel").await;

    assert_eq!(response.status, StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------------------------
// History
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn history_lists_sessions_of_the_range_newest_first(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let first = start_default(&app, &token).await;
    finish(&app, &token, &first, "complete").await;
    let second = start_default(&app, &token).await;
    finish(&app, &token, &second, "cancel").await;
    let third = start_default(&app, &token).await;
    let day = today(&db).await;

    let response = history(&app, &token, &format!("?from={day}&to={day}")).await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(ids(&response), [third, second, first]);
}

#[sqlx::test]
async fn history_filters_by_the_local_date(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let old = start_default(&app, &token).await;
    finish(&app, &token, &old, "complete").await;
    sqlx::query("UPDATE study_sessions SET local_date = '2026-01-15'")
        .execute(&db)
        .await
        .unwrap();
    let recent = start_default(&app, &token).await;
    let day = today(&db).await;

    let only_old = history(&app, &token, "?from=2026-01-15&to=2026-01-15").await;
    let only_recent = history(&app, &token, &format!("?from={day}&to={day}")).await;
    let both = history(&app, &token, &format!("?from=2026-01-01&to={day}")).await;

    assert_eq!(ids(&only_old), [old.as_str()]);
    assert_eq!(ids(&only_recent), [recent.as_str()]);
    assert_eq!(ids(&both).len(), 2);
}

#[sqlx::test]
async fn history_excludes_other_users(db: PgPool) {
    let app = common::app(db.clone());
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    start_default(&app, &grace).await;
    let day = today(&db).await;

    let response = history(&app, &ada, &format!("?from={day}&to={day}")).await;

    assert_eq!(response.status, StatusCode::OK);
    assert!(ids(&response).is_empty());
}

#[sqlx::test]
async fn history_requires_both_dates_in_order(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    for query in ["", "?from=2026-11-01", "?from=yesterday&to=2026-11-02"] {
        let response = history(&app, &token, query).await;

        assert_eq!(response.status, StatusCode::BAD_REQUEST, "{query}");
    }

    let reversed = history(&app, &token, "?from=2026-11-02&to=2026-11-01").await;
    assert_eq!(reversed.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        reversed.body["error"]["details"]["fields"]["to"],
        json!(["before_from"])
    );
}

// ---------------------------------------------------------------------------------------------
// Delete
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn delete_removes_the_session(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let id = start_default(&app, &token).await;
    finish(&app, &token, &id, "complete").await;
    let day = today(&db).await;

    let response = delete_session(&app, &token, &id).await;

    assert_eq!(response.status, StatusCode::NO_CONTENT);
    let listed = history(&app, &token, &format!("?from={day}&to={day}")).await;
    assert!(ids(&listed).is_empty());
}

#[sqlx::test]
async fn delete_hides_the_session_of_another_user(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let id = start_default(&app, &ada).await;

    let response = delete_session(&app, &grace, &id).await;

    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert_eq!(current(&app, &ada).await.status, StatusCode::OK);
}

#[sqlx::test]
async fn delete_answers_not_found_for_an_unknown_session(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = delete_session(&app, &token, "00000000-0000-4000-8000-000000000000").await;

    assert_eq!(response.status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn deleting_a_subject_keeps_its_sessions_without_a_subject(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let subject_id = create_subject(&app, &token, "Math").await;
    let started = start(&app, &token, json!({ "subject_id": subject_id })).await;
    let id = started.body["id"].as_str().unwrap();
    finish(&app, &token, id, "complete").await;
    let removed = TestRequest::new(Method::DELETE, &format!("/api/v1/subjects/{subject_id}"))
        .bearer(&token)
        .send(&app)
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    let day = today(&db).await;

    let response = history(&app, &token, &format!("?from={day}&to={day}")).await;

    assert_eq!(response.body[0]["id"], id);
    assert_eq!(response.body[0]["subject_id"], Value::Null);
}
