//! Integration tests for `GET /finances/summary`.

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

async fn create_category(app: &Router, token: &str, kind: &str, name: &str) -> String {
    let response = TestRequest::post("/api/v1/finance-categories")
        .bearer(token)
        .json(&json!({ "kind": kind, "name": name, "color": "#22C55E" }))
        .send(app)
        .await;
    assert_eq!(response.status, StatusCode::CREATED);
    response.body["id"].as_str().unwrap().to_owned()
}

async fn spend(app: &Router, token: &str, category: &str, amount: i64, date: &str) {
    let response = TestRequest::post("/api/v1/transactions")
        .bearer(token)
        .json(&json!({ "category_id": category, "amount": amount, "date": date }))
        .send(app)
        .await;
    assert_eq!(response.status, StatusCode::CREATED);
}

async fn set_budget(app: &Router, token: &str, budget: Value) {
    let response = TestRequest::new(Method::PATCH, "/api/v1/settings")
        .bearer(token)
        .json(&json!({ "monthly_budget": budget }))
        .send(app)
        .await;
    assert_eq!(response.status, StatusCode::OK);
}

async fn summary(app: &Router, token: &str, month: &str) -> TestResponse {
    TestRequest::get(&format!("/api/v1/finances/summary?month={month}"))
        .bearer(token)
        .send(app)
        .await
}

#[sqlx::test]
async fn an_empty_month_returns_zeros_and_no_budget(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = summary(&app, &token, "2026-03").await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body["income_total"], 0);
    assert_eq!(response.body["expense_total"], 0);
    assert_eq!(response.body["balance"], 0);
    assert_eq!(response.body["by_category"], json!([]));
    assert_eq!(response.body["budget"], Value::Null);
}

#[sqlx::test]
async fn totals_balance_and_categories_with_mixed_kinds(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let food = create_category(&app, &token, "expense", "Food").await;
    let bills = create_category(&app, &token, "expense", "Bills").await;
    let salary = create_category(&app, &token, "income", "Salary").await;
    // Created but unused in March: it must not appear in the breakdown.
    create_category(&app, &token, "expense", "Travel").await;
    spend(&app, &token, &food, 40_000, "2026-03-02").await;
    spend(&app, &token, &food, 10_000, "2026-03-20").await;
    spend(&app, &token, &bills, 70_000, "2026-03-31").await;
    spend(&app, &token, &salary, 500_000, "2026-03-01").await;
    // Neighbouring months are excluded.
    spend(&app, &token, &food, 999, "2026-02-28").await;
    spend(&app, &token, &salary, 999, "2026-04-01").await;

    let response = summary(&app, &token, "2026-03").await;

    assert_eq!(response.body["income_total"], 500_000);
    assert_eq!(response.body["expense_total"], 120_000);
    assert_eq!(response.body["balance"], 380_000);
    let rows: Vec<(String, String, i64)> = response.body["by_category"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["kind"].as_str().unwrap().to_owned(),
                row["category"]["name"].as_str().unwrap().to_owned(),
                row["total"].as_i64().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        [
            ("expense".to_owned(), "Bills".to_owned(), 70_000),
            ("expense".to_owned(), "Food".to_owned(), 50_000),
            ("income".to_owned(), "Salary".to_owned(), 500_000),
        ]
    );
    assert_eq!(response.body["by_category"][0]["category"]["id"], bills);
    assert_eq!(
        response.body["by_category"][0]["category"]["color"],
        "#22C55E"
    );
}

#[sqlx::test]
async fn the_balance_can_be_negative(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let food = create_category(&app, &token, "expense", "Food").await;
    spend(&app, &token, &food, 5_000, "2026-03-02").await;

    let response = summary(&app, &token, "2026-03").await;

    assert_eq!(response.body["balance"], -5_000);
}

#[sqlx::test]
async fn archived_categories_stay_in_the_breakdown(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let food = create_category(&app, &token, "expense", "Food").await;
    spend(&app, &token, &food, 5_000, "2026-03-02").await;
    TestRequest::new(Method::PATCH, &format!("/api/v1/finance-categories/{food}"))
        .bearer(&token)
        .json(&json!({ "archived": true }))
        .send(&app)
        .await;

    let response = summary(&app, &token, "2026-03").await;

    assert_eq!(response.body["by_category"].as_array().unwrap().len(), 1);
    assert_eq!(response.body["by_category"][0]["category"]["name"], "Food");
}

#[sqlx::test]
async fn the_budget_reports_the_amount_spent_in_the_month(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;
    let food = create_category(&app, &token, "expense", "Food").await;
    let salary = create_category(&app, &token, "income", "Salary").await;
    spend(&app, &token, &food, 40_000, "2026-03-02").await;
    spend(&app, &token, &salary, 900_000, "2026-03-03").await;
    set_budget(&app, &token, json!(100_000)).await;

    let with_budget = summary(&app, &token, "2026-03").await;
    let other_month = summary(&app, &token, "2026-04").await;
    set_budget(&app, &token, Value::Null).await;
    let cleared = summary(&app, &token, "2026-03").await;

    assert_eq!(
        with_budget.body["budget"],
        json!({ "amount": 100_000, "spent": 40_000 })
    );
    assert_eq!(
        other_month.body["budget"],
        json!({ "amount": 100_000, "spent": 0 })
    );
    assert_eq!(cleared.body["budget"], Value::Null);
}

#[sqlx::test]
async fn a_malformed_month_is_rejected(db: PgPool) {
    let app = common::app(db);
    let token = sign_up(&app, ADA).await;

    let response = summary(&app, &token, "2026-3").await;

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        response.body["error"]["details"]["fields"]["month"],
        json!(["format"])
    );
}

#[sqlx::test]
async fn another_users_data_is_invisible(db: PgPool) {
    let app = common::app(db);
    let ada = sign_up(&app, ADA).await;
    let grace = sign_up(&app, GRACE).await;
    let food = create_category(&app, &ada, "expense", "Food").await;
    spend(&app, &ada, &food, 40_000, "2026-03-02").await;
    set_budget(&app, &ada, json!(100_000)).await;

    let response = summary(&app, &grace, "2026-03").await;

    assert_eq!(response.body["expense_total"], 0);
    assert_eq!(response.body["by_category"], json!([]));
    assert_eq!(response.body["budget"], Value::Null);
}

#[sqlx::test]
async fn the_endpoint_requires_authentication(db: PgPool) {
    let app = common::app(db);

    let response = TestRequest::get("/api/v1/finances/summary?month=2026-03")
        .send(&app)
        .await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
}
