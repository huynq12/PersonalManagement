//! HTTP handlers for settings. The feature has no business rules beyond validation, so the
//! handlers call the repository directly.

use axum::{Json, extract::State};

use super::{model::PomodoroSettings, repository};
use crate::{AppState, auth::AuthUser, error::AppResult, validation::ValidatedJson};

/// `GET /api/v1/settings` — the Pomodoro settings.
pub(super) async fn get(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<PomodoroSettings>> {
    Ok(Json(repository::get_settings(&state.db, auth.id).await?))
}

/// `PUT /api/v1/settings` — replaces the Pomodoro settings.
pub(super) async fn replace(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(settings): ValidatedJson<PomodoroSettings>,
) -> AppResult<Json<PomodoroSettings>> {
    Ok(Json(
        repository::replace_settings(&state.db, auth.id, &settings).await?,
    ))
}
