//! HTTP handlers for the day view and day modes.

use axum::{Json, extract::State, http::StatusCode};
use chrono::NaiveDate;

use super::{
    model::{DayView, SetDayModeRequest},
    service,
};
use crate::{
    AppState,
    auth::AuthUser,
    error::AppResult,
    validation::{ApiPath, ValidatedJson},
};

/// `GET /api/v1/days/{date}` — the mode, the scheduled habits, and the to-dos of one date.
pub(super) async fn day_view(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(date): ApiPath<NaiveDate>,
) -> AppResult<Json<DayView>> {
    let view = service::day_view(&state.db, auth.id, date).await?;
    Ok(Json(view))
}

/// `PUT /api/v1/days/{date}/mode` — marks today as a normal or a low-energy day.
pub(super) async fn set_mode(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(date): ApiPath<NaiveDate>,
    ValidatedJson(request): ValidatedJson<SetDayModeRequest>,
) -> AppResult<StatusCode> {
    service::set_mode(&state.db, auth.id, date, request.mode).await?;
    Ok(StatusCode::NO_CONTENT)
}
