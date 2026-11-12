//! HTTP handlers for habits: extraction and response shaping only.

use axum::{Json, extract::State, http::StatusCode};
use chrono::NaiveDate;
use uuid::Uuid;

use super::{
    model::{CreateHabitRequest, Habit, ListHabitsQuery, UpdateHabitRequest},
    service,
};
use crate::{
    AppState,
    auth::AuthUser,
    error::AppResult,
    validation::{ApiPath, ApiQuery, ValidatedJson},
};

/// `GET /api/v1/habits` — the user's habits with streaks; `?include_archived=true` adds
/// archived ones.
pub(super) async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiQuery(query): ApiQuery<ListHabitsQuery>,
) -> AppResult<Json<Vec<Habit>>> {
    let habits = service::list(&state.db, auth.id, query.include_archived).await?;
    Ok(Json(habits))
}

/// `POST /api/v1/habits` — creates a habit.
pub(super) async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(request): ValidatedJson<CreateHabitRequest>,
) -> AppResult<(StatusCode, Json<Habit>)> {
    let habit = service::create(&state.db, auth.id, request).await?;
    Ok((StatusCode::CREATED, Json(habit)))
}

/// `PATCH /api/v1/habits/{id}` — updates name, weekdays, order, or archive state.
pub(super) async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(habit_id): ApiPath<Uuid>,
    ValidatedJson(request): ValidatedJson<UpdateHabitRequest>,
) -> AppResult<Json<Habit>> {
    let habit = service::update(&state.db, auth.id, habit_id, request).await?;
    Ok(Json(habit))
}

/// `DELETE /api/v1/habits/{id}` — deletes a habit with its history.
pub(super) async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(habit_id): ApiPath<Uuid>,
) -> AppResult<StatusCode> {
    service::delete(&state.db, auth.id, habit_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `PUT /api/v1/habits/{id}/completions/{date}` — marks the habit done on a day (idempotent).
pub(super) async fn complete(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath((habit_id, date)): ApiPath<(Uuid, NaiveDate)>,
) -> AppResult<StatusCode> {
    service::complete(&state.db, auth.id, habit_id, date).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /api/v1/habits/{id}/completions/{date}` — unmarks the habit on a day.
pub(super) async fn uncomplete(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath((habit_id, date)): ApiPath<(Uuid, NaiveDate)>,
) -> AppResult<StatusCode> {
    service::uncomplete(&state.db, auth.id, habit_id, date).await?;
    Ok(StatusCode::NO_CONTENT)
}
