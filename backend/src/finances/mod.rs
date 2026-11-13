//! Finances: income and expense categories, transactions, and the monthly summary.

use axum::{
    Router,
    routing::{get, patch},
};

use crate::AppState;

mod handler;
mod model;
mod repository;
mod service;

/// Routes of the finances feature, mounted under `/api/v1`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/finance-categories",
            get(handler::list_categories).post(handler::create_category),
        )
        .route(
            "/finance-categories/{id}",
            patch(handler::update_category).delete(handler::delete_category),
        )
        .route(
            "/transactions",
            get(handler::list_transactions).post(handler::create_transaction),
        )
        .route(
            "/transactions/{id}",
            patch(handler::update_transaction).delete(handler::delete_transaction),
        )
        .route("/finances/summary", get(handler::summary))
}
