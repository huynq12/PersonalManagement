//! Integration tests for the `/finance-categories` endpoints.

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
const GREEN: &str = "#22C55E";

async fn create_category(app: &Router, token: &str, body: Value) -> TestResponse {
    TestRequest::post("/api/v1/finance-categories")
        .bearer(token)
        .json(&body)
        .send(app)
        .await
}

/// Creates a category and returns its ID.
async fn create_named(app: &Router, token: &str, kind: &str, name: &str) -> String {
    let response = create_category(
        app,
        token,
        json!({ "kind": kind, "name": name, "color": GREEN }),
    )
    .await;
    assert_eq!(response.status, StatusCode::CREATED);
    response.body["id"].as_str().unwrap().to_owned()
}

async fn list_categories(app: &Router, token: &str, query: &str) -> TestResponse {
    TestRequest::get(&format!("/api/v1/finance-categories{query}"))
        .bearer(token)
        .send(app)
        .await
}

async fn patch_category(app: &Router, token: &str, id: &str, body: Value) -> TestResponse {
    TestRequest::new(Method::PATCH, &format!("/api/v1/finance-categories/{id}"))
        .bearer(token)
        .json(&body)
        .send(app)
        .await
}

async fn delete_category(app: &Router, token: &str, id: &str) -> TestResponse {
    TestRequest::new(Method::DELETE, &format!("/api/v1/finance-categories/{id}"))
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
        .map(|category| category["name"].as_str().unwrap())
        .collect()
}

fn error_code(response: &TestResponse) -> &str {
    response.body["error"]["code"].as_str().unwrap()
}

#[sqlx::test]
async fn create_returns_each_kind(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    for kind in ["expense", "income"] {
        let response = create_category(
            &app,
            &token,
            json!({ "kind": kind, "name": "Food", "color": GREEN }),
        )
        .await;

        assert_eq!(response.status, StatusCode::CREATED);
        assert_eq!(response.body["kind"], kind);
        assert_eq!(response.body["name"], "Food");
        assert_eq!(response.body["color"], GREEN);
        assert_eq!(response.body["archived_at"], Value::Null);
        assert!(response.body["id"].is_string());
    }
}

#[sqlx::test]
async fn create_trims_the_name(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = create_category(
        &app,
        &token,
        json!({ "kind": "expense", "name": "  Food ", "color": GREEN }),
    )
    .await;

    assert_eq!(response.body["name"], "Food");
}

#[sqlx::test]
async fn the_same_name_is_allowed_across_kinds_but_not_within_one(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    create_named(&app, &token, "expense", "Other").await;

    let other_kind = create_category(
        &app,
        &token,
        json!({ "kind": "income", "name": "Other", "color": GREEN }),
    )
    .await;
    let same_kind_other_case = create_category(
        &app,
        &token,
        json!({ "kind": "expense", "name": "OTHER", "color": GREEN }),
    )
    .await;

    assert_eq!(other_kind.status, StatusCode::CREATED);
    assert_eq!(same_kind_other_case.status, StatusCode::CONFLICT);
    assert_eq!(
        error_code(&same_kind_other_case),
        "finance_category_name_taken"
    );
}

#[sqlx::test]
async fn create_rejects_invalid_kind_color_and_name(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let bad_color = create_category(
        &app,
        &token,
        json!({ "kind": "expense", "name": "Food", "color": "green" }),
    )
    .await;
    let blank_name = create_category(
        &app,
        &token,
        json!({ "kind": "expense", "name": "  ", "color": GREEN }),
    )
    .await;
    let long_name = create_category(
        &app,
        &token,
        json!({ "kind": "expense", "name": "x".repeat(101), "color": GREEN }),
    )
    .await;
    let bad_kind = create_category(
        &app,
        &token,
        json!({ "kind": "gift", "name": "Food", "color": GREEN }),
    )
    .await;

    for response in [&bad_color, &blank_name, &long_name] {
        assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(error_code(response), "validation_failed");
    }
    // An unknown kind cannot be deserialized, so the body itself is malformed.
    assert_eq!(bad_kind.status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn list_orders_by_kind_then_name_and_filters_by_kind(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    create_named(&app, &token, "income", "Salary").await;
    create_named(&app, &token, "expense", "transport").await;
    create_named(&app, &token, "expense", "Food").await;

    let all = list_categories(&app, &token, "").await;
    let expenses = list_categories(&app, &token, "?kind=expense").await;
    let incomes = list_categories(&app, &token, "?kind=income").await;

    assert_eq!(names(&all), ["Food", "transport", "Salary"]);
    assert_eq!(names(&expenses), ["Food", "transport"]);
    assert_eq!(names(&incomes), ["Salary"]);
}

#[sqlx::test]
async fn archiving_hides_a_category_from_the_default_list(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = create_named(&app, &token, "expense", "Food").await;

    let archived = patch_category(&app, &token, &id, json!({ "archived": true })).await;
    let default_list = list_categories(&app, &token, "").await;
    let full_list = list_categories(&app, &token, "?include_archived=true").await;
    let restored = patch_category(&app, &token, &id, json!({ "archived": false })).await;

    assert_eq!(archived.status, StatusCode::OK);
    assert!(archived.body["archived_at"].is_string());
    assert!(names(&default_list).is_empty());
    assert_eq!(names(&full_list), ["Food"]);
    assert_eq!(restored.body["archived_at"], Value::Null);
}

#[sqlx::test]
async fn an_archived_name_stays_taken(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = create_named(&app, &token, "expense", "Food").await;
    patch_category(&app, &token, &id, json!({ "archived": true })).await;

    let again = create_category(
        &app,
        &token,
        json!({ "kind": "expense", "name": "food", "color": GREEN }),
    )
    .await;

    assert_eq!(again.status, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn patch_changes_name_and_color_but_not_kind(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = create_named(&app, &token, "expense", "Food").await;
    create_named(&app, &token, "expense", "Bills").await;

    let renamed = patch_category(
        &app,
        &token,
        &id,
        json!({ "name": "Groceries", "color": "#EF4444" }),
    )
    .await;
    let taken = patch_category(&app, &token, &id, json!({ "name": "bills" })).await;
    let bad_color = patch_category(&app, &token, &id, json!({ "color": "red" })).await;
    let kind_change = patch_category(&app, &token, &id, json!({ "kind": "income" })).await;

    assert_eq!(renamed.status, StatusCode::OK);
    assert_eq!(renamed.body["name"], "Groceries");
    assert_eq!(renamed.body["color"], "#EF4444");
    assert_eq!(renamed.body["kind"], "expense");
    assert_eq!(taken.status, StatusCode::CONFLICT);
    assert_eq!(error_code(&taken), "finance_category_name_taken");
    assert_eq!(bad_color.status, StatusCode::UNPROCESSABLE_ENTITY);
    // The kind is not an accepted field; it is ignored and never changes.
    assert_eq!(kind_change.status, StatusCode::OK);
    assert_eq!(kind_change.body["kind"], "expense");
}

#[sqlx::test]
async fn delete_removes_an_unused_category(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = create_named(&app, &token, "expense", "Food").await;

    let deleted = delete_category(&app, &token, &id).await;
    let again = delete_category(&app, &token, &id).await;

    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    assert_eq!(again.status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn delete_of_a_used_category_is_a_conflict(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let id = create_named(&app, &token, "expense", "Food").await;
    let transaction = TestRequest::post("/api/v1/transactions")
        .bearer(&token)
        .json(&json!({ "category_id": id, "amount": 5000, "date": "2026-01-10" }))
        .send(&app)
        .await;
    assert_eq!(transaction.status, StatusCode::CREATED);

    let response = delete_category(&app, &token, &id).await;

    assert_eq!(response.status, StatusCode::CONFLICT);
    assert_eq!(error_code(&response), "finance_category_in_use");
}

#[sqlx::test]
async fn another_users_category_is_not_found(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let id = create_named(&app, &ada, "expense", "Food").await;

    let listed = list_categories(&app, &grace, "?include_archived=true").await;
    let patched = patch_category(&app, &grace, &id, json!({ "name": "Mine" })).await;
    let deleted = delete_category(&app, &grace, &id).await;

    assert!(names(&listed).is_empty());
    assert_eq!(patched.status, StatusCode::NOT_FOUND);
    assert_eq!(deleted.status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn endpoints_require_authentication(db: PgPool) {
    let app = common::app(db);

    let response = TestRequest::get("/api/v1/finance-categories")
        .send(&app)
        .await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
}
