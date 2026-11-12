//! Integration tests for the subjects endpoints.

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
const BLUE: &str = "#3B82F6";

async fn create_subject(app: &Router, token: &str, body: Value) -> TestResponse {
    TestRequest::post("/api/v1/subjects")
        .bearer(token)
        .json(&body)
        .send(app)
        .await
}

/// Creates a subject and returns its ID.
async fn create_named(app: &Router, token: &str, name: &str) -> String {
    let response = create_subject(app, token, json!({ "name": name, "color": BLUE })).await;
    assert_eq!(response.status, StatusCode::CREATED);
    response.body["id"].as_str().unwrap().to_owned()
}

async fn list_subjects(app: &Router, token: &str, query: &str) -> TestResponse {
    TestRequest::get(&format!("/api/v1/subjects{query}"))
        .bearer(token)
        .send(app)
        .await
}

async fn patch_subject(app: &Router, token: &str, id: &str, body: Value) -> TestResponse {
    TestRequest::new(Method::PATCH, &format!("/api/v1/subjects/{id}"))
        .bearer(token)
        .json(&body)
        .send(app)
        .await
}

async fn delete_subject(app: &Router, token: &str, id: &str) -> TestResponse {
    TestRequest::new(Method::DELETE, &format!("/api/v1/subjects/{id}"))
        .bearer(token)
        .send(app)
        .await
}

fn names(response: &TestResponse) -> Vec<&str> {
    response
        .body
        .as_array()
        .unwrap()
        .iter()
        .map(|subject| subject["name"].as_str().unwrap())
        .collect()
}

fn error_code(response: &TestResponse) -> &str {
    response.body["error"]["code"].as_str().unwrap()
}

// ---------------------------------------------------------------------------------------------
// Create
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn create_returns_the_new_subject(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = create_subject(&app, &token, json!({ "name": "Math", "color": BLUE })).await;

    assert_eq!(response.status, StatusCode::CREATED);
    assert_eq!(response.body["name"], "Math");
    assert_eq!(response.body["color"], BLUE);
    assert_eq!(response.body["archived_at"], Value::Null);
    assert!(response.body["id"].is_string());
    assert!(response.body["created_at"].is_string());
}

#[sqlx::test]
async fn create_trims_the_name(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = create_subject(&app, &token, json!({ "name": "  Math  ", "color": BLUE })).await;

    assert_eq!(response.status, StatusCode::CREATED);
    assert_eq!(response.body["name"], "Math");
}

#[sqlx::test]
async fn create_accepts_lowercase_hex_color(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response =
        create_subject(&app, &token, json!({ "name": "Math", "color": "#3b82f6" })).await;

    assert_eq!(response.status, StatusCode::CREATED);
}

#[sqlx::test]
async fn create_rejects_a_blank_or_too_long_name(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let blank = create_subject(&app, &token, json!({ "name": "   ", "color": BLUE })).await;
    assert_eq!(blank.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(error_code(&blank), "validation_failed");
    assert_eq!(
        blank.body["error"]["details"]["fields"]["name"],
        json!(["blank"])
    );

    let long_name = "a".repeat(101);
    let long = create_subject(&app, &token, json!({ "name": long_name, "color": BLUE })).await;
    assert_eq!(long.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        long.body["error"]["details"]["fields"]["name"],
        json!(["length"])
    );
}

#[sqlx::test]
async fn create_rejects_a_color_that_is_not_rrggbb(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    for color in ["3B82F6", "#3B8", "#3B82F6FF", "#GGGGGG", "blue", ""] {
        let response =
            create_subject(&app, &token, json!({ "name": "Math", "color": color })).await;

        assert_eq!(
            response.status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "color {color:?}"
        );
        assert_eq!(error_code(&response), "validation_failed");
        assert_eq!(
            response.body["error"]["details"]["fields"]["color"],
            json!(["color"]),
            "color {color:?}"
        );
    }
}

#[sqlx::test]
async fn create_rejects_a_missing_field(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = create_subject(&app, &token, json!({ "name": "Math" })).await;

    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&response), "invalid_request");
}

#[sqlx::test]
async fn create_rejects_a_duplicate_name_ignoring_case(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    create_named(&app, &token, "Math").await;

    let response = create_subject(&app, &token, json!({ "name": "mATH", "color": BLUE })).await;

    assert_eq!(response.status, StatusCode::CONFLICT);
    assert_eq!(error_code(&response), "subject_name_taken");
}

#[sqlx::test]
async fn create_rejects_a_name_used_by_an_archived_subject(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = create_named(&app, &token, "Math").await;
    let archived = patch_subject(&app, &token, &id, json!({ "archived": true })).await;
    assert_eq!(archived.status, StatusCode::OK);

    let response = create_subject(&app, &token, json!({ "name": "Math", "color": BLUE })).await;

    assert_eq!(response.status, StatusCode::CONFLICT);
    assert_eq!(error_code(&response), "subject_name_taken");
}

#[sqlx::test]
async fn create_allows_the_same_name_for_different_users(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    create_named(&app, &ada, "Math").await;

    let response = create_subject(&app, &grace, json!({ "name": "Math", "color": BLUE })).await;

    assert_eq!(response.status, StatusCode::CREATED);
}

#[sqlx::test]
async fn create_requires_authentication(db: PgPool) {
    let app = common::app(db);

    let response = TestRequest::post("/api/v1/subjects")
        .json(&json!({ "name": "Math", "color": BLUE }))
        .send(&app)
        .await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn list_is_empty_for_a_new_user(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = list_subjects(&app, &token, "").await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body, json!([]));
}

#[sqlx::test]
async fn list_sorts_by_name_ignoring_case(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    for name in ["physics", "Biology", "chemistry", "Art"] {
        create_named(&app, &token, name).await;
    }

    let response = list_subjects(&app, &token, "").await;

    assert_eq!(names(&response), ["Art", "Biology", "chemistry", "physics"]);
}

#[sqlx::test]
async fn list_hides_archived_subjects_unless_asked(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    create_named(&app, &token, "Art").await;
    let math = create_named(&app, &token, "Math").await;
    patch_subject(&app, &token, &math, json!({ "archived": true })).await;

    let default = list_subjects(&app, &token, "").await;
    assert_eq!(names(&default), ["Art"]);

    let explicit_false = list_subjects(&app, &token, "?include_archived=false").await;
    assert_eq!(names(&explicit_false), ["Art"]);

    let all = list_subjects(&app, &token, "?include_archived=true").await;
    assert_eq!(names(&all), ["Art", "Math"]);
    assert!(all.body[1]["archived_at"].is_string());
}

#[sqlx::test]
async fn list_rejects_a_malformed_query(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = list_subjects(&app, &token, "?include_archived=maybe").await;

    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&response), "invalid_request");
}

#[sqlx::test]
async fn list_only_returns_the_users_own_subjects(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    create_named(&app, &ada, "Math").await;
    create_named(&app, &grace, "History").await;

    let response = list_subjects(&app, &ada, "?include_archived=true").await;

    assert_eq!(names(&response), ["Math"]);
}

#[sqlx::test]
async fn list_requires_authentication(db: PgPool) {
    let app = common::app(db);

    let response = TestRequest::get("/api/v1/subjects").send(&app).await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------------------------
// Update
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn update_changes_name_and_color(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = create_named(&app, &token, "Math").await;

    let response = patch_subject(
        &app,
        &token,
        &id,
        json!({ "name": " Algebra ", "color": "#EF4444" }),
    )
    .await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["id"], id.as_str());
    assert_eq!(response.body["name"], "Algebra");
    assert_eq!(response.body["color"], "#EF4444");
}

#[sqlx::test]
async fn update_keeps_fields_that_are_left_out(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = create_named(&app, &token, "Math").await;

    let response = patch_subject(&app, &token, &id, json!({})).await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["name"], "Math");
    assert_eq!(response.body["color"], BLUE);
    assert_eq!(response.body["archived_at"], Value::Null);
}

#[sqlx::test]
async fn update_archives_and_restores(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = create_named(&app, &token, "Math").await;

    let archived = patch_subject(&app, &token, &id, json!({ "archived": true })).await;
    assert_eq!(archived.status, StatusCode::OK);
    let archived_at = archived.body["archived_at"].clone();
    assert!(archived_at.is_string());

    // Archiving twice keeps the first archive time.
    let again = patch_subject(&app, &token, &id, json!({ "archived": true })).await;
    assert_eq!(again.body["archived_at"], archived_at);

    let restored = patch_subject(&app, &token, &id, json!({ "archived": false })).await;
    assert_eq!(restored.status, StatusCode::OK);
    assert_eq!(restored.body["archived_at"], Value::Null);
}

#[sqlx::test]
async fn update_allows_changing_only_the_case_of_the_own_name(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = create_named(&app, &token, "math").await;

    let response = patch_subject(&app, &token, &id, json!({ "name": "Math" })).await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["name"], "Math");
}

#[sqlx::test]
async fn update_rejects_invalid_values(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = create_named(&app, &token, "Math").await;

    let blank = patch_subject(&app, &token, &id, json!({ "name": " " })).await;
    assert_eq!(blank.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        blank.body["error"]["details"]["fields"]["name"],
        json!(["blank"])
    );

    let color = patch_subject(&app, &token, &id, json!({ "color": "red" })).await;
    assert_eq!(color.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        color.body["error"]["details"]["fields"]["color"],
        json!(["color"])
    );

    let unchanged = list_subjects(&app, &token, "").await;
    assert_eq!(unchanged.body[0]["name"], "Math");
    assert_eq!(unchanged.body[0]["color"], BLUE);
}

#[sqlx::test]
async fn update_rejects_a_name_taken_by_another_subject(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    create_named(&app, &token, "Math").await;
    let art = create_named(&app, &token, "Art").await;

    let response = patch_subject(&app, &token, &art, json!({ "name": "MATH" })).await;

    assert_eq!(response.status, StatusCode::CONFLICT);
    assert_eq!(error_code(&response), "subject_name_taken");
}

#[sqlx::test]
async fn update_rejects_a_name_taken_by_an_archived_subject(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let math = create_named(&app, &token, "Math").await;
    patch_subject(&app, &token, &math, json!({ "archived": true })).await;
    let art = create_named(&app, &token, "Art").await;

    let response = patch_subject(&app, &token, &art, json!({ "name": "Math" })).await;

    assert_eq!(response.status, StatusCode::CONFLICT);
    assert_eq!(error_code(&response), "subject_name_taken");
}

#[sqlx::test]
async fn update_of_another_users_subject_is_not_found(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let id = create_named(&app, &ada, "Math").await;

    let response = patch_subject(&app, &grace, &id, json!({ "name": "Stolen" })).await;

    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&response), "not_found");
    let own = list_subjects(&app, &ada, "").await;
    assert_eq!(names(&own), ["Math"]);
}

#[sqlx::test]
async fn update_of_an_unknown_subject_is_not_found(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = patch_subject(
        &app,
        &token,
        "00000000-0000-0000-0000-000000000000",
        json!({ "name": "Math" }),
    )
    .await;

    assert_eq!(response.status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn update_rejects_a_malformed_id(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = patch_subject(&app, &token, "not-a-uuid", json!({ "name": "Math" })).await;

    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&response), "invalid_request");
}

// ---------------------------------------------------------------------------------------------
// Delete
// ---------------------------------------------------------------------------------------------

#[sqlx::test]
async fn delete_removes_the_subject(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = create_named(&app, &token, "Math").await;

    let response = delete_subject(&app, &token, &id).await;

    assert_eq!(response.status, StatusCode::NO_CONTENT);
    let remaining = list_subjects(&app, &token, "?include_archived=true").await;
    assert_eq!(remaining.body, json!([]));
}

#[sqlx::test]
async fn delete_frees_the_name_for_reuse(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = create_named(&app, &token, "Math").await;
    delete_subject(&app, &token, &id).await;

    let response = create_subject(&app, &token, json!({ "name": "Math", "color": BLUE })).await;

    assert_eq!(response.status, StatusCode::CREATED);
}

#[sqlx::test]
async fn delete_twice_is_not_found(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = create_named(&app, &token, "Math").await;
    delete_subject(&app, &token, &id).await;

    let response = delete_subject(&app, &token, &id).await;

    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&response), "not_found");
}

#[sqlx::test]
async fn delete_of_another_users_subject_is_not_found(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let id = create_named(&app, &ada, "Math").await;

    let response = delete_subject(&app, &grace, &id).await;

    assert_eq!(response.status, StatusCode::NOT_FOUND);
    let own = list_subjects(&app, &ada, "").await;
    assert_eq!(names(&own), ["Math"]);
}

#[sqlx::test]
async fn delete_requires_authentication(db: PgPool) {
    let app = common::app(db);

    let response = TestRequest::new(
        Method::DELETE,
        "/api/v1/subjects/00000000-0000-0000-0000-000000000000",
    )
    .send(&app)
    .await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
}
