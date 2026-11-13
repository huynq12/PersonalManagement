//! HTTP handlers for settings: extraction and response shaping only.

use axum::{Json, extract::State};

use super::{
    model::{Settings, UpdateSettingsRequest},
    service,
};
use crate::{AppState, auth::AuthUser, error::AppResult, validation::ValidatedJson};

/// `GET /api/v1/settings` — the user's settings.
pub(super) async fn get(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Settings>> {
    let settings = service::get(&state.db, auth.id).await?;
    Ok(Json(settings))
}

/// `PATCH /api/v1/settings` — updates any subset of the settings.
pub(super) async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(request): ValidatedJson<UpdateSettingsRequest>,
) -> AppResult<Json<Settings>> {
    let settings = service::update(&state.db, auth.id, request).await?;
    Ok(Json(settings))
}
