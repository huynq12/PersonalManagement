//! HTTP handler for the weekly statistics: extraction and response shaping only.

use axum::{Json, extract::State};

use super::{
    model::{WeeklyStats, WeeklyStatsQuery},
    service,
};
use crate::{AppState, auth::AuthUser, error::AppResult, validation::ApiQuery};

/// `GET /api/v1/stats/weekly?week_start=` — the statistics of one week and its comparison with
/// the week before. Without `week_start` the current week is used.
pub(super) async fn weekly(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiQuery(query): ApiQuery<WeeklyStatsQuery>,
) -> AppResult<Json<WeeklyStats>> {
    let stats = service::weekly(&state.db, auth.id, query.week_start).await?;
    Ok(Json(stats))
}
