//! Integration tests for the `/transactions` endpoints.

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

async fn create_category(app: &Router, token: &str, kind: &str, name: &str) -> String {
    let response = TestRequest::post("/api/v1/finance-categories")
        .bearer(token)
        .json(&json!({ "kind": kind, "name": name, "color": GREEN }))
        .send(app)
        .await;
    assert_eq!(response.status, StatusCode::CREATED);
    response.body["id"].as_str().unwrap().to_owned()
}

async fn create_transaction(app: &Router, token: &str, body: Value) -> TestResponse {
    TestRequest::post("/api/v1/transactions")
        .bearer(token)
        .json(&body)
        .send(app)
        .await
}

/// Creates a transaction and returns its ID.
async fn create_on(app: &Router, token: &str, category: &str, amount: i64, date: &str) -> String {
    let response = create_transaction(
        app,
        token,
        json!({ "category_id": category, "amount": amount, "date": date }),
    )
    .await;
    assert_eq!(response.status, StatusCode::CREATED);
    response.body["id"].as_str().unwrap().to_owned()
}

async fn list(app: &Router, token: &str, month: &str) -> TestResponse {
    TestRequest::get(&format!("/api/v1/transactions?month={month}"))
        .bearer(token)
        .send(app)
        .await
}

async fn patch_transaction(app: &Router, token: &str, id: &str, body: Value) -> TestResponse {
    TestRequest::new(Method::PATCH, &format!("/api/v1/transactions/{id}"))
        .bearer(token)
        .json(&body)
        .send(app)
        .await
}

async fn delete_transaction(app: &Router, token: &str, id: &str) -> TestResponse {
    TestRequest::new(Method::DELETE, &format!("/api/v1/transactions/{id}"))
        .bearer(token)
        .send(app)
        .await
}

async fn archive(app: &Router, token: &str, category: &str) {
    let response = TestRequest::new(
        Method::PATCH,
        &format!("/api/v1/finance-categories/{category}"),
    )
    .bearer(token)
    .json(&json!({ "archived": true }))
    .send(app)
    .await;
    assert_eq!(response.status, StatusCode::OK);
}

fn field_codes<'a>(response: &'a TestResponse, field: &str) -> Vec<&'a str> {
    response.body["error"]["details"]["fields"][field]
        .as_array()
        .unwrap()
        .iter()
        .map(|code| code.as_str().unwrap())
        .collect()
}

fn amounts(response: &TestResponse) -> Vec<i64> {
    response
        .body
        .as_array()
        .unwrap()
        .iter()
        .map(|transaction| transaction["amount"].as_i64().unwrap())
        .collect()
}

#[sqlx::test]
async fn create_returns_the_transaction_with_its_category(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let food = create_category(&app, &token, "expense", "Food").await;

    let response = create_transaction(
        &app,
        &token,
        json!({ "category_id": food, "amount": 45_000, "date": "2026-03-05", "note": " lunch " }),
    )
    .await;

    assert_eq!(response.status, StatusCode::CREATED);
    assert_eq!(response.body["amount"], 45_000);
    assert_eq!(response.body["date"], "2026-03-05");
    assert_eq!(response.body["note"], "lunch");
    assert_eq!(response.body["kind"], "expense");
    assert_eq!(response.body["category"]["id"], food);
    assert_eq!(response.body["category"]["name"], "Food");
    assert_eq!(response.body["category"]["color"], GREEN);
    assert!(response.body["id"].is_string());
}

#[sqlx::test]
async fn create_accepts_past_and_future_dates(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let food = create_category(&app, &token, "expense", "Food").await;

    for date in ["2001-01-01", "2999-12-31"] {
        let response = create_transaction(
            &app,
            &token,
            json!({ "category_id": food, "amount": 1, "date": date }),
        )
        .await;
        assert_eq!(response.status, StatusCode::CREATED, "{date}");
    }
}

#[sqlx::test]
async fn create_rejects_a_non_positive_amount_and_a_long_note(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let food = create_category(&app, &token, "expense", "Food").await;

    for amount in [0, -5] {
        let response = create_transaction(
            &app,
            &token,
            json!({ "category_id": food, "amount": amount, "date": "2026-03-05" }),
        )
        .await;
        assert_eq!(
            response.status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{amount}"
        );
        assert_eq!(field_codes(&response, "amount"), ["range"]);
    }
    let long_note = create_transaction(
        &app,
        &token,
        json!({
            "category_id": food, "amount": 1, "date": "2026-03-05", "note": "x".repeat(501),
        }),
    )
    .await;
    assert_eq!(long_note.status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn create_rejects_an_unknown_or_foreign_category(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let adas_food = create_category(&app, &ada, "expense", "Food").await;

    let foreign = create_transaction(
        &app,
        &grace,
        json!({ "category_id": adas_food, "amount": 1, "date": "2026-03-05" }),
    )
    .await;
    let unknown = create_transaction(
        &app,
        &grace,
        json!({
            "category_id": "00000000-0000-0000-0000-000000000000",
            "amount": 1,
            "date": "2026-03-05",
        }),
    )
    .await;

    for response in [&foreign, &unknown] {
        assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(field_codes(response, "category_id"), ["not_found"]);
    }
}

#[sqlx::test]
async fn create_rejects_an_archived_category(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let food = create_category(&app, &token, "expense", "Food").await;
    archive(&app, &token, &food).await;

    let response = create_transaction(
        &app,
        &token,
        json!({ "category_id": food, "amount": 1, "date": "2026-03-05" }),
    )
    .await;

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(field_codes(&response, "category_id"), ["archived"]);
}

#[sqlx::test]
async fn list_covers_one_month_newest_date_then_newest_created_first(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let food = create_category(&app, &token, "expense", "Food").await;
    create_on(&app, &token, &food, 1, "2026-02-28").await;
    create_on(&app, &token, &food, 2, "2026-03-01").await;
    create_on(&app, &token, &food, 3, "2026-03-31").await;
    create_on(&app, &token, &food, 4, "2026-03-31").await;
    create_on(&app, &token, &food, 5, "2026-04-01").await;

    let march = list(&app, &token, "2026-03").await;
    let empty = list(&app, &token, "2026-05").await;

    assert_eq!(march.status, StatusCode::OK);
    assert_eq!(amounts(&march), [4, 3, 2]);
    assert!(amounts(&empty).is_empty());
}

#[sqlx::test]
async fn list_rejects_a_malformed_month(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    for month in ["2026-13", "2026", "March"] {
        let response = list(&app, &token, month).await;
        assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY, "{month}");
        assert_eq!(field_codes(&response, "month"), ["format"]);
    }
}

#[sqlx::test]
async fn list_includes_the_kind_of_each_transaction(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let food = create_category(&app, &token, "expense", "Food").await;
    let salary = create_category(&app, &token, "income", "Salary").await;
    create_on(&app, &token, &food, 1, "2026-03-01").await;
    create_on(&app, &token, &salary, 2, "2026-03-02").await;

    let response = list(&app, &token, "2026-03").await;

    let kinds: Vec<&str> = response
        .body
        .as_array()
        .unwrap()
        .iter()
        .map(|transaction| transaction["kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds, ["income", "expense"]);
}

#[sqlx::test]
async fn patch_updates_amount_date_and_note(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let food = create_category(&app, &token, "expense", "Food").await;
    let id = create_on(&app, &token, &food, 100, "2026-03-05").await;

    let response = patch_transaction(
        &app,
        &token,
        &id,
        json!({ "amount": 250, "date": "2026-04-02", "note": "dinner" }),
    )
    .await;
    let invalid = patch_transaction(&app, &token, &id, json!({ "amount": 0 })).await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["amount"], 250);
    assert_eq!(response.body["date"], "2026-04-02");
    assert_eq!(response.body["note"], "dinner");
    assert_eq!(invalid.status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn patch_can_clear_the_note(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let food = create_category(&app, &token, "expense", "Food").await;
    let created = create_transaction(
        &app,
        &token,
        json!({ "category_id": food, "amount": 1, "date": "2026-03-05", "note": "lunch" }),
    )
    .await;
    let id = created.body["id"].as_str().unwrap();

    let cleared = patch_transaction(&app, &token, id, json!({ "note": null })).await;
    let kept = patch_transaction(&app, &token, id, json!({ "amount": 2 })).await;

    assert_eq!(cleared.body["note"], Value::Null);
    assert_eq!(kept.body["note"], Value::Null);
}

#[sqlx::test]
async fn patch_moves_to_a_category_of_the_same_kind_only(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let food = create_category(&app, &token, "expense", "Food").await;
    let bills = create_category(&app, &token, "expense", "Bills").await;
    let salary = create_category(&app, &token, "income", "Salary").await;
    let id = create_on(&app, &token, &food, 100, "2026-03-05").await;

    let moved = patch_transaction(&app, &token, &id, json!({ "category_id": bills })).await;
    let mismatch = patch_transaction(&app, &token, &id, json!({ "category_id": salary })).await;
    let unknown = patch_transaction(
        &app,
        &token,
        &id,
        json!({ "category_id": "00000000-0000-0000-0000-000000000000" }),
    )
    .await;

    assert_eq!(moved.status, StatusCode::OK);
    assert_eq!(moved.body["category"]["name"], "Bills");
    assert_eq!(mismatch.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(field_codes(&mismatch, "category_id"), ["kind_mismatch"]);
    assert_eq!(unknown.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(field_codes(&unknown, "category_id"), ["not_found"]);
}

#[sqlx::test]
async fn patch_rejects_an_archived_category_only_when_it_changes(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let food = create_category(&app, &token, "expense", "Food").await;
    let bills = create_category(&app, &token, "expense", "Bills").await;
    let id = create_on(&app, &token, &food, 100, "2026-03-05").await;
    archive(&app, &token, &food).await;
    archive(&app, &token, &bills).await;

    let same_category = patch_transaction(
        &app,
        &token,
        &id,
        json!({ "category_id": food, "amount": 200 }),
    )
    .await;
    let amount_only = patch_transaction(&app, &token, &id, json!({ "amount": 300 })).await;
    let to_archived = patch_transaction(&app, &token, &id, json!({ "category_id": bills })).await;

    assert_eq!(same_category.status, StatusCode::OK);
    assert_eq!(amount_only.status, StatusCode::OK);
    assert_eq!(to_archived.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(field_codes(&to_archived, "category_id"), ["archived"]);
}

#[sqlx::test]
async fn delete_removes_the_transaction(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let food = create_category(&app, &token, "expense", "Food").await;
    let id = create_on(&app, &token, &food, 100, "2026-03-05").await;

    let deleted = delete_transaction(&app, &token, &id).await;
    let again = delete_transaction(&app, &token, &id).await;
    let remaining = list(&app, &token, "2026-03").await;

    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    assert_eq!(again.status, StatusCode::NOT_FOUND);
    assert!(amounts(&remaining).is_empty());
}

#[sqlx::test]
async fn another_users_transactions_are_invisible(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let food = create_category(&app, &ada, "expense", "Food").await;
    let id = create_on(&app, &ada, &food, 100, "2026-03-05").await;

    let listed = list(&app, &grace, "2026-03").await;
    let patched = patch_transaction(&app, &grace, &id, json!({ "amount": 1 })).await;
    let deleted = delete_transaction(&app, &grace, &id).await;
    let still_there = list(&app, &ada, "2026-03").await;

    assert!(amounts(&listed).is_empty());
    assert_eq!(patched.status, StatusCode::NOT_FOUND);
    assert_eq!(deleted.status, StatusCode::NOT_FOUND);
    assert_eq!(amounts(&still_there), [100]);
}

#[sqlx::test]
async fn endpoints_require_authentication(db: PgPool) {
    let app = common::app(db);

    let response = TestRequest::get("/api/v1/transactions?month=2026-03")
        .send(&app)
        .await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
}
