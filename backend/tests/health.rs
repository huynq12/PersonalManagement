//! Integration tests for the health endpoint, the 404 fallback, and the initial migration.

// Test helpers are not `#[test]` functions, so the clippy.toml test exemption does not reach them.
#![allow(clippy::unwrap_used)]
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;

mod common;

async fn get(db: PgPool, uri: &str) -> (StatusCode, Value) {
    let app = common::app(db);
    let response = app
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();

    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap())
}

// `sqlx::test` creates a temporary database per test and applies the migrations.
#[sqlx::test]
async fn health_returns_ok(db: PgPool) {
    let (status, body) = get(db, "/api/v1/health").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
    assert_eq!(body["database"], "ok");
}

#[sqlx::test]
async fn unknown_route_returns_json_404(db: PgPool) {
    let (status, body) = get(db, "/api/v1/does-not-exist").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");
    assert_eq!(body["error"]["message"], "resource not found");
}

#[sqlx::test]
async fn migrations_enable_citext(db: PgPool) {
    let equal: bool = sqlx::query_scalar("SELECT 'A@B.com'::citext = 'a@b.COM'::citext")
        .fetch_one(&db)
        .await
        .unwrap();

    assert!(equal);
}
