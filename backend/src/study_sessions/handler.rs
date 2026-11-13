//! HTTP handlers for study sessions: extraction and response shaping only.

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use super::{
    model::{ListSessionsQuery, StartSessionRequest, StudySession},
    service,
};
use crate::{
    AppState,
    auth::AuthUser,
    error::AppResult,
    validation::{ApiPath, ApiQuery, ValidatedJson},
};

/// `POST /api/v1/study-sessions` — starts a focus session.
pub(super) async fn start(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(request): ValidatedJson<StartSessionRequest>,
) -> AppResult<(StatusCode, Json<StudySession>)> {
    let session = service::start(&state.db, auth.id, request.subject_id).await?;
    Ok((StatusCode::CREATED, Json(session)))
}

/// `GET /api/v1/study-sessions/current` — the running session, or 204 if there is none.
pub(super) async fn current(State(state): State<AppState>, auth: AuthUser) -> AppResult<Response> {
    let response = match service::current(&state.db, auth.id).await? {
        Some(session) => Json(session).into_response(),
        None => StatusCode::NO_CONTENT.into_response(),
    };
    Ok(response)
}

/// `POST /api/v1/study-sessions/{id}/complete` — finishes the session.
pub(super) async fn complete(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(session_id): ApiPath<Uuid>,
) -> AppResult<Json<StudySession>> {
    let session = service::complete(&state.db, auth.id, session_id).await?;
    Ok(Json(session))
}

/// `POST /api/v1/study-sessions/{id}/cancel` — cancels the session.
pub(super) async fn cancel(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(session_id): ApiPath<Uuid>,
) -> AppResult<Json<StudySession>> {
    let session = service::cancel(&state.db, auth.id, session_id).await?;
    Ok(Json(session))
}

/// `GET /api/v1/study-sessions?from=&to=` — session history for a range of local dates.
pub(super) async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiQuery(query): ApiQuery<ListSessionsQuery>,
) -> AppResult<Json<Vec<StudySession>>> {
    let sessions = service::list(&state.db, auth.id, query.from, query.to).await?;
    Ok(Json(sessions))
}

/// `DELETE /api/v1/study-sessions/{id}` — deletes a session.
pub(super) async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(session_id): ApiPath<Uuid>,
) -> AppResult<StatusCode> {
    service::delete(&state.db, auth.id, session_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
