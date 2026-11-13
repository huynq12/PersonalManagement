//! HTTP handlers for the work log: extraction and response shaping only.

use axum::{Json, extract::State, http::StatusCode};
use chrono::NaiveDate;

use super::{
    model::{ListQuery, MonthView, PutWorkEntryRequest, WorkEntry},
    service,
};
use crate::{
    AppState,
    auth::AuthUser,
    error::AppResult,
    validation::{ApiPath, ApiQuery, ValidatedJson},
};

/// `GET /api/v1/work-entries?month=YYYY-MM` — the month's entries and totals.
pub(super) async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiQuery(query): ApiQuery<ListQuery>,
) -> AppResult<Json<MonthView>> {
    let view = service::list(&state.db, auth.id, &query.month).await?;
    Ok(Json(view))
}

/// `PUT /api/v1/work-entries/{date}` — creates or updates the day's entry.
pub(super) async fn put(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(date): ApiPath<NaiveDate>,
    ValidatedJson(request): ValidatedJson<PutWorkEntryRequest>,
) -> AppResult<Json<WorkEntry>> {
    let entry = service::put(&state.db, auth.id, date, request).await?;
    Ok(Json(entry))
}

/// `DELETE /api/v1/work-entries/{date}` — deletes the day's entry.
pub(super) async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(date): ApiPath<NaiveDate>,
) -> AppResult<StatusCode> {
    service::delete(&state.db, auth.id, date).await?;
    Ok(StatusCode::NO_CONTENT)
}
