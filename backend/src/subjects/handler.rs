//! HTTP handlers for subjects: extraction and response shaping only.

use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use super::{
    model::{CreateSubjectRequest, ListSubjectsQuery, Subject, UpdateSubjectRequest},
    service,
};
use crate::{
    AppState,
    auth::AuthUser,
    error::AppResult,
    validation::{ApiPath, ApiQuery, ValidatedJson},
};

/// `GET /api/v1/subjects` — the user's subjects by name; `?include_archived=true` adds
/// archived ones.
pub(super) async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiQuery(query): ApiQuery<ListSubjectsQuery>,
) -> AppResult<Json<Vec<Subject>>> {
    let subjects = service::list(&state.db, auth.id, query.include_archived).await?;
    Ok(Json(subjects))
}

/// `POST /api/v1/subjects` — creates a subject.
pub(super) async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(request): ValidatedJson<CreateSubjectRequest>,
) -> AppResult<(StatusCode, Json<Subject>)> {
    let subject = service::create(&state.db, auth.id, request).await?;
    Ok((StatusCode::CREATED, Json(subject)))
}

/// `PATCH /api/v1/subjects/{id}` — updates name, color, or archive state.
pub(super) async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(subject_id): ApiPath<Uuid>,
    ValidatedJson(request): ValidatedJson<UpdateSubjectRequest>,
) -> AppResult<Json<Subject>> {
    let subject = service::update(&state.db, auth.id, subject_id, request).await?;
    Ok(Json(subject))
}

/// `DELETE /api/v1/subjects/{id}` — deletes a subject.
pub(super) async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(subject_id): ApiPath<Uuid>,
) -> AppResult<StatusCode> {
    service::delete(&state.db, auth.id, subject_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
