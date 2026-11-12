//! HTTP handlers for the Pomodoro settings: extraction and response shaping only.

use axum::{Json, extract::State};

use super::{model::Settings, service};
use crate::{AppState, auth::AuthUser, error::AppResult, validation::ValidatedJson};

/// `GET /api/v1/settings` — the user's Pomodoro settings.
pub(super) async fn get(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Settings>> {
    Ok(Json(service::get(&state.db, auth.id).await?))
}

/// `PUT /api/v1/settings` — replaces the user's Pomodoro settings.
pub(super) async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(settings): ValidatedJson<Settings>,
) -> AppResult<Json<Settings>> {
    Ok(Json(service::update(&state.db, auth.id, settings).await?))
}
