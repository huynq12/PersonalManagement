//! HTTP handler for the day view.

use axum::{Json, extract::State};
use chrono::NaiveDate;

use super::{model::DayView, service};
use crate::{AppState, auth::AuthUser, error::AppResult, validation::ApiPath};

/// `GET /api/v1/days/{date}` — the scheduled habits and the to-dos of one date.
pub(super) async fn day_view(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(date): ApiPath<NaiveDate>,
) -> AppResult<Json<DayView>> {
    let view = service::day_view(&state.db, auth.id, date).await?;
    Ok(Json(view))
}
