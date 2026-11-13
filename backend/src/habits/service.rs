//! Business rules for habits and their completions.

use chrono::{NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    model::{CreateHabitRequest, Habit, UpdateHabitRequest, normalize_weekdays},
    repository::{self, HabitChanges},
};
use crate::{
    error::{AppError, AppResult},
    validation::field_error,
};

/// Every ISO weekday: the schedule of a habit created without one.
const EVERY_DAY: [i16; 7] = [1, 2, 3, 4, 5, 6, 7];

/// Lists the user's habits with their streaks.
pub(super) async fn list(
    db: &PgPool,
    user_id: Uuid,
    include_archived: bool,
) -> AppResult<Vec<Habit>> {
    Ok(repository::list_habits(db, user_id, include_archived, Utc::now()).await?)
}

/// Creates a habit, scheduled every day unless weekdays are given.
pub(super) async fn create(
    db: &PgPool,
    user_id: Uuid,
    request: CreateHabitRequest,
) -> AppResult<Habit> {
    let weekdays = request
        .weekdays
        .map_or_else(|| EVERY_DAY.to_vec(), normalize_weekdays);

    let low_energy = request.low_energy.unwrap_or(false);
    // A habit that is not kept has no easier version; a blank one means "use the normal name".
    let low_energy_name = request
        .low_energy_name
        .as_deref()
        .map(str::trim)
        .filter(|name| low_energy && !name.is_empty());

    let habit = repository::insert_habit(
        db,
        user_id,
        request.name.trim(),
        &weekdays,
        low_energy,
        low_energy_name,
        Utc::now(),
    )
    .await?;
    Ok(habit)
}

/// Updates the given fields of a habit.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if the user has no such habit.
pub(super) async fn update(
    db: &PgPool,
    user_id: Uuid,
    habit_id: Uuid,
    request: UpdateHabitRequest,
) -> AppResult<Habit> {
    let weekdays = request.weekdays.map(normalize_weekdays);
    let changes = HabitChanges {
        name: request.name.as_deref().map(str::trim),
        weekdays: weekdays.as_deref(),
        low_energy: request.low_energy,
        low_energy_name: request.low_energy_name.as_deref().map(str::trim),
        sort_order: request.sort_order,
        archived: request.archived,
    };

    repository::update_habit(db, user_id, habit_id, changes, Utc::now())
        .await?
        .ok_or(AppError::NotFound)
}

/// Deletes a habit together with its history.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if the user has no such habit.
pub(super) async fn delete(db: &PgPool, user_id: Uuid, habit_id: Uuid) -> AppResult<()> {
    if repository::delete_habit(db, user_id, habit_id).await? {
        Ok(())
    } else {
        Err(AppError::NotFound)
    }
}

/// Marks a habit as done on `date`. Doing so twice is not an error.
///
/// Only today and past days the habit is scheduled for can be completed: a future completion
/// is meaningless, and one on an unscheduled day would never count towards the streak.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if the user has no such habit, and
/// [`AppError::Validation`] on the `date` field with code `future` or `not_scheduled`.
pub(super) async fn complete(
    db: &PgPool,
    user_id: Uuid,
    habit_id: Uuid,
    date: NaiveDate,
) -> AppResult<()> {
    let now = Utc::now();
    let check = repository::check_completion_date(db, user_id, habit_id, date, now)
        .await?
        .ok_or(AppError::NotFound)?;

    if check.is_future {
        return Err(field_error("date", "future"));
    }
    if !check.is_scheduled {
        return Err(field_error("date", "not_scheduled"));
    }

    repository::insert_completion(db, habit_id, date, now).await?;
    Ok(())
}

/// Removes the completion of a habit on `date`. Removing a missing one is not an error.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if the user has no such habit.
pub(super) async fn uncomplete(
    db: &PgPool,
    user_id: Uuid,
    habit_id: Uuid,
    date: NaiveDate,
) -> AppResult<()> {
    if !repository::habit_exists(db, user_id, habit_id).await? {
        return Err(AppError::NotFound);
    }
    repository::delete_completion(db, user_id, habit_id, date).await?;
    Ok(())
}
