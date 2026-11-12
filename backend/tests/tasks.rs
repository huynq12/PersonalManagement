//! Integration tests for to-dos.

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
const DATE: &str = "2026-11-12";
const OTHER_DATE: &str = "2026-11-13";

async fn create_task(app: &Router, token: &str, body: Value) -> TestResponse {
    TestRequest::post("/api/v1/tasks")
        .bearer(token)
        .json(&body)
        .send(app)
        .await
}

/// Creates a to-do on [`DATE`] and returns its ID.
async fn add_task(app: &Router, token: &str, title: &str) -> String {
    let response = create_task(app, token, json!({ "date": DATE, "title": title })).await;
    assert_eq!(response.status, StatusCode::CREATED);
    response.body["id"].as_str().unwrap().to_owned()
}

async fn list_tasks(app: &Router, token: &str, date: &str) -> TestResponse {
    TestRequest::get(&format!("/api/v1/tasks?date={date}"))
        .bearer(token)
        .send(app)
        .await
}

async fn patch_task(app: &Router, token: &str, id: &str, body: Value) -> TestResponse {
    TestRequest::new(Method::PATCH, &format!("/api/v1/tasks/{id}"))
        .bearer(token)
        .json(&body)
        .send(app)
        .await
}

async fn delete_task(app: &Router, token: &str, id: &str) -> TestResponse {
    TestRequest::new(Method::DELETE, &format!("/api/v1/tasks/{id}"))
        .bearer(token)
        .send(app)
        .await
}

fn titles(response: &TestResponse) -> Vec<&str> {
    response
        .body
        .as_array()
        .unwrap()
        .iter()
        .map(|task| task["title"].as_str().unwrap())
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Create
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn create_returns_an_open_task_on_the_given_date(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = create_task(&app, &token, json!({ "date": DATE, "title": "Buy milk" })).await;

    assert_eq!(response.status, StatusCode::CREATED);
    assert_eq!(response.body["title"], "Buy milk");
    assert_eq!(response.body["date"], DATE);
    assert_eq!(response.body["is_done"], false);
    assert_eq!(response.body["done_at"], Value::Null);
    assert!(response.body["id"].is_string());
}

#[sqlx::test]
async fn create_trims_the_title(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = create_task(
        &app,
        &token,
        json!({ "date": DATE, "title": "  Buy milk " }),
    )
    .await;

    assert_eq!(response.body["title"], "Buy milk");
}

#[sqlx::test]
async fn create_appends_tasks_in_the_order_they_were_added(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    add_task(&app, &token, "First").await;
    add_task(&app, &token, "Second").await;

    let tasks = list_tasks(&app, &token, DATE).await;

    assert_eq!(titles(&tasks), ["First", "Second"]);
}

#[sqlx::test]
async fn create_rejects_blank_title(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = create_task(&app, &token, json!({ "date": DATE, "title": "  " })).await;

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        response.body["error"]["details"]["fields"]["title"],
        json!(["blank"])
    );
}

#[sqlx::test]
async fn create_rejects_title_longer_than_200_characters(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = create_task(
        &app,
        &token,
        json!({ "date": DATE, "title": "a".repeat(201) }),
    )
    .await;

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        response.body["error"]["details"]["fields"]["title"],
        json!(["length"])
    );
}

#[sqlx::test]
async fn create_rejects_missing_or_malformed_date(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let missing = create_task(&app, &token, json!({ "title": "Buy milk" })).await;
    let malformed = create_task(
        &app,
        &token,
        json!({ "date": "12/11/2026", "title": "Buy milk" }),
    )
    .await;

    assert_eq!(missing.status, StatusCode::BAD_REQUEST);
    assert_eq!(malformed.status, StatusCode::BAD_REQUEST);
    assert_eq!(malformed.body["error"]["code"], "invalid_request");
}

#[sqlx::test]
async fn create_requires_authentication(db: PgPool) {
    let app = common::app(db);

    let response = TestRequest::post("/api/v1/tasks")
        .json(&json!({ "date": DATE, "title": "Buy milk" }))
        .send(&app)
        .await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn list_returns_only_tasks_of_the_requested_date(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    add_task(&app, &token, "Today").await;
    create_task(
        &app,
        &token,
        json!({ "date": OTHER_DATE, "title": "Tomorrow" }),
    )
    .await;

    let tasks = list_tasks(&app, &token, DATE).await;

    assert_eq!(tasks.status, StatusCode::OK);
    assert_eq!(titles(&tasks), ["Today"]);
}

#[sqlx::test]
async fn list_returns_only_the_tasks_of_the_signed_in_user(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    add_task(&app, &ada, "Ada's task").await;
    add_task(&app, &grace, "Grace's task").await;

    let tasks = list_tasks(&app, &ada, DATE).await;

    assert_eq!(titles(&tasks), ["Ada's task"]);
}

#[sqlx::test]
async fn list_requires_a_valid_date(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let missing = TestRequest::get("/api/v1/tasks")
        .bearer(&token)
        .send(&app)
        .await;
    let malformed = list_tasks(&app, &token, "yesterday").await;

    assert_eq!(missing.status, StatusCode::BAD_REQUEST);
    assert_eq!(malformed.status, StatusCode::BAD_REQUEST);
    assert_eq!(malformed.body["error"]["code"], "invalid_request");
}

// ---------------------------------------------------------------------------------------------
// Update
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn update_marks_a_task_done_and_records_when(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = add_task(&app, &token, "Buy milk").await;

    let response = patch_task(&app, &token, &id, json!({ "is_done": true })).await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["is_done"], true);
    assert!(response.body["done_at"].is_string());
    assert_eq!(response.body["title"], "Buy milk");
}

#[sqlx::test]
async fn update_keeps_the_first_completion_time_when_marked_done_again(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = add_task(&app, &token, "Buy milk").await;
    let first = patch_task(&app, &token, &id, json!({ "is_done": true })).await;

    let second = patch_task(&app, &token, &id, json!({ "is_done": true })).await;

    assert_eq!(second.body["done_at"], first.body["done_at"]);
}

#[sqlx::test]
async fn update_reopens_a_task_and_clears_its_completion_time(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = add_task(&app, &token, "Buy milk").await;
    patch_task(&app, &token, &id, json!({ "is_done": true })).await;

    let response = patch_task(&app, &token, &id, json!({ "is_done": false })).await;

    assert_eq!(response.body["is_done"], false);
    assert_eq!(response.body["done_at"], Value::Null);
}

#[sqlx::test]
async fn update_moves_a_task_to_another_date(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = add_task(&app, &token, "Buy milk").await;

    patch_task(&app, &token, &id, json!({ "date": OTHER_DATE })).await;

    assert!(titles(&list_tasks(&app, &token, DATE).await).is_empty());
    assert_eq!(
        titles(&list_tasks(&app, &token, OTHER_DATE).await),
        ["Buy milk"]
    );
}

#[sqlx::test]
async fn update_changes_title_and_order(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    add_task(&app, &token, "First").await;
    let second = add_task(&app, &token, "Second").await;

    patch_task(
        &app,
        &token,
        &second,
        json!({ "title": " Renamed ", "sort_order": -1 }),
    )
    .await;

    assert_eq!(
        titles(&list_tasks(&app, &token, DATE).await),
        ["Renamed", "First"]
    );
}

#[sqlx::test]
async fn update_rejects_blank_title(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = add_task(&app, &token, "Buy milk").await;

    let response = patch_task(&app, &token, &id, json!({ "title": "" })).await;

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        response.body["error"]["details"]["fields"]["title"],
        json!(["blank"])
    );
}

#[sqlx::test]
async fn update_of_another_users_task_is_not_found(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let id = add_task(&app, &ada, "Buy milk").await;

    let response = patch_task(&app, &grace, &id, json!({ "is_done": true })).await;

    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert_eq!(response.body["error"]["code"], "not_found");
    assert_eq!(list_tasks(&app, &ada, DATE).await.body[0]["is_done"], false);
}

#[sqlx::test]
async fn update_rejects_malformed_id(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = patch_task(&app, &token, "42", json!({ "is_done": true })).await;

    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(response.body["error"]["code"], "invalid_request");
}

// ---------------------------------------------------------------------------------------------
// Delete
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn delete_removes_the_task(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = add_task(&app, &token, "Buy milk").await;

    let response = delete_task(&app, &token, &id).await;

    assert_eq!(response.status, StatusCode::NO_CONTENT);
    assert!(titles(&list_tasks(&app, &token, DATE).await).is_empty());
}

#[sqlx::test]
async fn delete_of_a_missing_task_is_not_found(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = add_task(&app, &token, "Buy milk").await;
    delete_task(&app, &token, &id).await;

    let response = delete_task(&app, &token, &id).await;

    assert_eq!(response.status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn delete_of_another_users_task_is_not_found(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let id = add_task(&app, &ada, "Buy milk").await;

    let response = delete_task(&app, &grace, &id).await;

    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert_eq!(titles(&list_tasks(&app, &ada, DATE).await), ["Buy milk"]);
}
