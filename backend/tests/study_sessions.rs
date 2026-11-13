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

async fn start(app: &Router, token: &str, body: Value) -> TestResponse {
    TestRequest::post("/api/v1/study-sessions")
        .bearer(token)
        .json(&body)
        .send(app)
        .await
}

/// Starts a session without a subject and returns its ID.
async fn start_plain(app: &Router, token: &str) -> String {
    let response = start(app, token, json!({})).await;
    assert_eq!(response.status, StatusCode::CREATED);
    response.body["id"].as_str().unwrap().to_owned()
}

async fn create_subject(app: &Router, token: &str, name: &str) -> String {
    let response = TestRequest::post("/api/v1/subjects")
        .bearer(token)
        .json(&json!({ "name": name, "color": "#3B82F6" }))
        .send(app)
        .await;
    assert_eq!(response.status, StatusCode::CREATED);
    response.body["id"].as_str().unwrap().to_owned()
}

async fn action(app: &Router, token: &str, id: &str, verb: &str) -> TestResponse {
    TestRequest::post(&format!("/api/v1/study-sessions/{id}/{verb}"))
        .bearer(token)
        .send(app)
        .await
}

async fn current(app: &Router, token: &str) -> TestResponse {
    TestRequest::get("/api/v1/study-sessions/current")
        .bearer(token)
        .send(app)
        .await
}

async fn list(app: &Router, token: &str, query: &str) -> TestResponse {
    TestRequest::get(&format!("/api/v1/study-sessions{query}"))
        .bearer(token)
        .send(app)
        .await
}

async fn delete(app: &Router, token: &str, id: &str) -> TestResponse {
    TestRequest::new(Method::DELETE, &format!("/api/v1/study-sessions/{id}"))
        .bearer(token)
        .send(app)
        .await
}

fn error_code(response: &TestResponse) -> &str {
    response.body["error"]["code"].as_str().unwrap()
}

/// Moves a session's start into the past so the server sees time having elapsed.
async fn backdate(db: &PgPool, id: &str, seconds: i32) {
    sqlx::query(
        "UPDATE study_sessions
         SET started_at = now() - make_interval(secs => $2)
         WHERE id = $1::uuid",
    )
    .bind(id)
    .bind(f64::from(seconds))
    .execute(db)
    .await
    .unwrap();
}

// ---------------------------------------------------------------------------------------------
// Start
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn start_returns_a_running_session(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let subject_id = create_subject(&app, &token, "Rust").await;

    let response = start(&app, &token, json!({ "subject_id": subject_id })).await;

    assert_eq!(response.status, StatusCode::CREATED);
    assert_eq!(response.body["status"], "running");
    assert_eq!(response.body["subject_id"], subject_id);
    assert_eq!(response.body["planned_minutes"], 25);
    assert_eq!(response.body["ended_at"], Value::Null);
    assert_eq!(response.body["duration_seconds"], Value::Null);
    assert_eq!(
        response.body["local_date"],
        today(&db).await.to_string().as_str()
    );
}

#[sqlx::test]
async fn start_uses_the_focus_length_from_user_settings(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    sqlx::query("UPDATE user_settings SET focus_minutes = 40")
        .execute(&db)
        .await
        .unwrap();

    let response = start(&app, &token, json!({})).await;

    assert_eq!(response.status, StatusCode::CREATED);
    assert_eq!(response.body["planned_minutes"], 40);
    assert_eq!(response.body["subject_id"], Value::Null);
}

#[sqlx::test]
async fn start_with_another_users_subject_is_not_found(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let subject_id = create_subject(&app, &ada, "Rust").await;

    let response = start(&app, &grace, json!({ "subject_id": subject_id })).await;

    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&response), "not_found");
}

#[sqlx::test]
async fn start_while_a_session_is_running_conflicts(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    start_plain(&app, &token).await;

    let response = start(&app, &token, json!({})).await;

    assert_eq!(response.status, StatusCode::CONFLICT);
    assert_eq!(error_code(&response), "study_session_already_running");
}

#[sqlx::test]
async fn start_by_two_users_does_not_conflict(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    start_plain(&app, &ada).await;

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
async fn current_returns_the_running_session(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = start_plain(&app, &token).await;

    let response = current(&app, &token).await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["id"], id);
}

#[sqlx::test]
async fn current_without_a_running_session_is_no_content(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = current(&app, &token).await;

    assert_eq!(response.status, StatusCode::NO_CONTENT);
}

#[sqlx::test]
async fn current_ignores_another_users_session(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    start_plain(&app, &ada).await;

    let response = current(&app, &grace).await;

    assert_eq!(response.status, StatusCode::NO_CONTENT);
}

// ---------------------------------------------------------------------------------------------
// Complete
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn complete_records_the_elapsed_time(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let id = start_plain(&app, &token).await;
    backdate(&db, &id, 600).await;

    let response = action(&app, &token, &id, "complete").await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["status"], "completed");
    assert_ne!(response.body["ended_at"], Value::Null);
    let seconds = response.body["duration_seconds"].as_i64().unwrap();
    assert!((600..620).contains(&seconds), "duration was {seconds}");
}

#[sqlx::test]
async fn complete_caps_the_duration_at_the_planned_length(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let id = start_plain(&app, &token).await;
    backdate(&db, &id, 2 * 60 * 60).await;

    let response = action(&app, &token, &id, "complete").await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["duration_seconds"], 25 * 60);
}

#[sqlx::test]
async fn complete_frees_the_user_to_start_another_session(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = start_plain(&app, &token).await;
    action(&app, &token, &id, "complete").await;

    assert_eq!(current(&app, &token).await.status, StatusCode::NO_CONTENT);
    assert_eq!(
        start(&app, &token, json!({})).await.status,
        StatusCode::CREATED
    );
}

#[sqlx::test]
async fn complete_twice_conflicts(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = start_plain(&app, &token).await;
    action(&app, &token, &id, "complete").await;

    let response = action(&app, &token, &id, "complete").await;

    assert_eq!(response.status, StatusCode::CONFLICT);
    assert_eq!(error_code(&response), "study_session_not_running");
}

#[sqlx::test]
async fn complete_another_users_session_is_not_found(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let id = start_plain(&app, &ada).await;

    let response = action(&app, &grace, &id, "complete").await;

    assert_eq!(response.status, StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------------------------
// Cancel
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn cancel_marks_the_session_cancelled(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = start_plain(&app, &token).await;

    let response = action(&app, &token, &id, "cancel").await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["status"], "cancelled");
    assert_ne!(response.body["ended_at"], Value::Null);
    assert_eq!(response.body["duration_seconds"], Value::Null);
    assert_eq!(current(&app, &token).await.status, StatusCode::NO_CONTENT);
}

#[sqlx::test]
async fn cancel_a_finished_session_conflicts(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = start_plain(&app, &token).await;
    action(&app, &token, &id, "complete").await;

    let response = action(&app, &token, &id, "cancel").await;

    assert_eq!(response.status, StatusCode::CONFLICT);
    assert_eq!(error_code(&response), "study_session_not_running");
}

#[sqlx::test]
async fn cancel_another_users_session_is_not_found(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let id = start_plain(&app, &ada).await;

    let response = action(&app, &grace, &id, "cancel").await;

    assert_eq!(response.status, StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------------------------
// History
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn list_returns_sessions_in_the_date_range_newest_first(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let old = start_plain(&app, &token).await;
    action(&app, &token, &old, "cancel").await;
    let recent = start_plain(&app, &token).await;
    action(&app, &token, &recent, "cancel").await;
    sqlx::query(
        "UPDATE study_sessions
         SET local_date = local_date - 10, started_at = started_at - interval '10 days'
         WHERE id = $1::uuid",
    )
    .bind(&old)
    .execute(&db)
    .await
    .unwrap();
    let day = today(&db).await;

    let in_range = list(&app, &token, &format!("?from={day}&to={day}")).await;
    let wide = list(
        &app,
        &token,
        &format!("?from={}&to={day}", day - chrono::Days::new(30)),
    )
    .await;

    assert_eq!(in_range.status, StatusCode::OK);
    let ids = |response: &TestResponse| -> Vec<String> {
        response
            .body
            .as_array()
            .unwrap()
            .iter()
            .map(|session| session["id"].as_str().unwrap().to_owned())
            .collect()
    };
    assert_eq!(ids(&in_range), vec![recent.clone()]);
    assert_eq!(ids(&wide), vec![recent, old]);
}

#[sqlx::test]
async fn list_excludes_another_users_sessions(db: PgPool) {
    let app = common::app(db.clone());
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    start_plain(&app, &ada).await;
    let day = today(&db).await;

    let response = list(&app, &grace, &format!("?from={day}&to={day}")).await;

    assert_eq!(response.status, StatusCode::OK);
    assert!(response.body.as_array().unwrap().is_empty());
}

#[sqlx::test]
async fn list_without_a_range_is_rejected(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = list(&app, &token, "").await;

    assert_eq!(response.status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn list_with_an_inverted_range_is_rejected(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = list(&app, &token, "?from=2026-11-10&to=2026-11-01").await;

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(error_code(&response), "validation_failed");
}

// ---------------------------------------------------------------------------------------------
// Delete
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn delete_removes_the_session(db: PgPool) {
    let app = common::app(db.clone());
    let token = sign_up(&app, ADA).await;
    let id = start_plain(&app, &token).await;
    action(&app, &token, &id, "complete").await;

    let response = delete(&app, &token, &id).await;

    assert_eq!(response.status, StatusCode::NO_CONTENT);
    let day = today(&db).await;
    let history = list(&app, &token, &format!("?from={day}&to={day}")).await;
    assert!(history.body.as_array().unwrap().is_empty());
}

#[sqlx::test]
async fn delete_another_users_session_is_not_found(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let id = start_plain(&app, &ada).await;

    let response = delete(&app, &grace, &id).await;

    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert_eq!(current(&app, &ada).await.status, StatusCode::OK);
}

#[sqlx::test]
async fn deleting_a_subject_keeps_its_sessions_without_a_subject(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let subject_id = create_subject(&app, &token, "Rust").await;
    let started = start(&app, &token, json!({ "subject_id": subject_id })).await;
    let id = started.body["id"].as_str().unwrap();

    let deleted = TestRequest::new(Method::DELETE, &format!("/api/v1/subjects/{subject_id}"))
        .bearer(&token)
        .send(&app)
        .await;

    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    let response = current(&app, &token).await;
    assert_eq!(response.body["id"], id);
    assert_eq!(response.body["subject_id"], Value::Null);
}
