//! HTTP handlers for to-dos: extraction and response shaping only.

use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use super::{
    model::{CreateTaskRequest, ListTasksQuery, Task, UpdateTaskRequest},
    service,
};
use crate::{
    AppState,
    auth::AuthUser,
    error::AppResult,
    validation::{ApiPath, ApiQuery, ValidatedJson},
};

/// `GET /api/v1/tasks?date=YYYY-MM-DD` — the user's to-dos for one day.
pub(super) async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiQuery(query): ApiQuery<ListTasksQuery>,
) -> AppResult<Json<Vec<Task>>> {
    let tasks = service::list(&state.db, auth.id, query.date).await?;
    Ok(Json(tasks))
}

/// `POST /api/v1/tasks` — creates a to-do.
pub(super) async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(request): ValidatedJson<CreateTaskRequest>,
) -> AppResult<(StatusCode, Json<Task>)> {
    let task = service::create(&state.db, auth.id, request).await?;
    Ok((StatusCode::CREATED, Json(task)))
}

/// `PATCH /api/v1/tasks/{id}` — updates title, state, date, or order.
pub(super) async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(task_id): ApiPath<Uuid>,
    ValidatedJson(request): ValidatedJson<UpdateTaskRequest>,
) -> AppResult<Json<Task>> {
    let task = service::update(&state.db, auth.id, task_id, request).await?;
    Ok(Json(task))
}

/// `DELETE /api/v1/tasks/{id}` — deletes a to-do.
pub(super) async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(task_id): ApiPath<Uuid>,
) -> AppResult<StatusCode> {
    service::delete(&state.db, auth.id, task_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
