//! HTTP handlers for finances: extraction and response shaping only.

use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use super::{
    model::{
        Category, CreateCategoryRequest, CreateTransactionRequest, ListCategoriesQuery, MonthQuery,
        Summary, Transaction, UpdateCategoryRequest, UpdateTransactionRequest,
    },
    service,
};
use crate::{
    AppState,
    auth::AuthUser,
    error::AppResult,
    validation::{ApiPath, ApiQuery, ValidatedJson},
};

/// `GET /api/v1/finance-categories` — the user's categories; `?kind=` filters by kind and
/// `?include_archived=true` adds archived ones.
pub(super) async fn list_categories(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiQuery(query): ApiQuery<ListCategoriesQuery>,
) -> AppResult<Json<Vec<Category>>> {
    Ok(Json(
        service::list_categories(&state.db, auth.id, query).await?,
    ))
}

/// `POST /api/v1/finance-categories` — creates a category.
pub(super) async fn create_category(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(request): ValidatedJson<CreateCategoryRequest>,
) -> AppResult<(StatusCode, Json<Category>)> {
    let category = service::create_category(&state.db, auth.id, request).await?;
    Ok((StatusCode::CREATED, Json(category)))
}

/// `PATCH /api/v1/finance-categories/{id}` — updates name, color, or archive state.
pub(super) async fn update_category(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(category_id): ApiPath<Uuid>,
    ValidatedJson(request): ValidatedJson<UpdateCategoryRequest>,
) -> AppResult<Json<Category>> {
    let category = service::update_category(&state.db, auth.id, category_id, request).await?;
    Ok(Json(category))
}

/// `DELETE /api/v1/finance-categories/{id}` — deletes a category nothing uses.
pub(super) async fn delete_category(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(category_id): ApiPath<Uuid>,
) -> AppResult<StatusCode> {
    service::delete_category(&state.db, auth.id, category_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `GET /api/v1/transactions?month=` — the month's transactions.
pub(super) async fn list_transactions(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiQuery(query): ApiQuery<MonthQuery>,
) -> AppResult<Json<Vec<Transaction>>> {
    let transactions = service::list_transactions(&state.db, auth.id, &query.month).await?;
    Ok(Json(transactions))
}

/// `POST /api/v1/transactions` — records an income or an expense.
pub(super) async fn create_transaction(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(request): ValidatedJson<CreateTransactionRequest>,
) -> AppResult<(StatusCode, Json<Transaction>)> {
    let transaction = service::create_transaction(&state.db, auth.id, request).await?;
    Ok((StatusCode::CREATED, Json(transaction)))
}

/// `PATCH /api/v1/transactions/{id}` — updates category, amount, date, or note.
pub(super) async fn update_transaction(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(transaction_id): ApiPath<Uuid>,
    ValidatedJson(request): ValidatedJson<UpdateTransactionRequest>,
) -> AppResult<Json<Transaction>> {
    let transaction =
        service::update_transaction(&state.db, auth.id, transaction_id, request).await?;
    Ok(Json(transaction))
}

/// `DELETE /api/v1/transactions/{id}` — deletes a transaction.
pub(super) async fn delete_transaction(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(transaction_id): ApiPath<Uuid>,
) -> AppResult<StatusCode> {
    service::delete_transaction(&state.db, auth.id, transaction_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `GET /api/v1/finances/summary?month=` — the month's totals, breakdown, and budget use.
pub(super) async fn summary(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiQuery(query): ApiQuery<MonthQuery>,
) -> AppResult<Json<Summary>> {
    Ok(Json(
        service::summary(&state.db, auth.id, &query.month).await?,
    ))
}
