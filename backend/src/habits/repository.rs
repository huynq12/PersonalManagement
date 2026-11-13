//! SQL for habits and their completions.
//!
//! Every query filters by `user_id`, so one user can never read or change another user's
//! habits. Streaks are computed by the `habit_current_streak` SQL function (see the checklist
//! migration); `now` is passed in so a request uses a single clock reading.

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use super::model::Habit;

/// Lists a user's habits in display order.
pub(super) async fn list_habits(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    include_archived: bool,
    now: DateTime<Utc>,
) -> sqlx::Result<Vec<Habit>> {
    sqlx::query_as!(
        Habit,
        r#"
        SELECT id, name, weekdays, low_energy, low_energy_name, sort_order, archived_at, created_at,
               habit_current_streak(id, user_id, weekdays, low_energy, user_today($1, $3)) AS "current_streak!"
        FROM habits
        WHERE user_id = $1 AND ($2 OR archived_at IS NULL)
        ORDER BY sort_order, created_at
        "#,
        user_id,
        include_archived,
        now,
    )
    .fetch_all(executor)
    .await
}

/// Inserts a habit after the user's existing ones.
pub(super) async fn insert_habit(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    name: &str,
    weekdays: &[i16],
    low_energy: bool,
    low_energy_name: Option<&str>,
    now: DateTime<Utc>,
) -> sqlx::Result<Habit> {
    sqlx::query_as!(
        Habit,
        r#"
        INSERT INTO habits (user_id, name, weekdays, low_energy, low_energy_name, sort_order)
        VALUES (
            $1, $2, $3, $4, $5,
            (SELECT COALESCE(max(sort_order) + 1, 0) FROM habits WHERE user_id = $1)
        )
        RETURNING id, name, weekdays, low_energy, low_energy_name, sort_order, archived_at,
                  created_at,
                  habit_current_streak(id, user_id, weekdays, low_energy, user_today($1, $6)) AS "current_streak!"
        "#,
        user_id,
        name,
        weekdays,
        low_energy,
        low_energy_name,
        now,
    )
    .fetch_one(executor)
    .await
}

/// Changes of `PATCH /habits/{id}`; `None` keeps the current value.
pub(super) struct HabitChanges<'a> {
    pub name: Option<&'a str>,
    pub weekdays: Option<&'a [i16]>,
    pub low_energy: Option<bool>,
    /// Trimmed; empty clears the easier name.
    pub low_energy_name: Option<&'a str>,
    pub sort_order: Option<i32>,
    pub archived: Option<bool>,
}

/// Updates a habit. Returns `None` if the user has no such habit.
pub(super) async fn update_habit(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    habit_id: Uuid,
    changes: HabitChanges<'_>,
    now: DateTime<Utc>,
) -> sqlx::Result<Option<Habit>> {
    sqlx::query_as!(
        Habit,
        r#"
        UPDATE habits
        SET name = COALESCE($3, name),
            weekdays = COALESCE($4, weekdays),
            low_energy = COALESCE($5, low_energy),
            low_energy_name = CASE
                -- A habit that is not kept has no easier version.
                WHEN NOT COALESCE($5, low_energy) THEN NULL
                WHEN $6::text IS NULL THEN low_energy_name
                ELSE NULLIF($6, '')
            END,
            sort_order = COALESCE($7, sort_order),
            archived_at = CASE
                WHEN $8::boolean IS NULL THEN archived_at
                -- Archiving twice keeps the first archive time.
                WHEN $8 THEN COALESCE(archived_at, $9)
                ELSE NULL
            END
        WHERE id = $2 AND user_id = $1
        RETURNING id, name, weekdays, low_energy, low_energy_name, sort_order, archived_at,
                  created_at,
                  habit_current_streak(id, user_id, weekdays, low_energy, user_today($1, $9)) AS "current_streak!"
        "#,
        user_id,
        habit_id,
        changes.name,
        changes.weekdays,
        changes.low_energy,
        changes.low_energy_name,
        changes.sort_order,
        changes.archived,
        now,
    )
    .fetch_optional(executor)
    .await
}

/// Deletes a habit and, through the foreign key, its completions. Returns whether it existed.
pub(super) async fn delete_habit(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    habit_id: Uuid,
) -> sqlx::Result<bool> {
    let result = sqlx::query!(
        "DELETE FROM habits WHERE id = $2 AND user_id = $1",
        user_id,
        habit_id,
    )
    .execute(executor)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// How a date relates to a habit's schedule and to the user's current day.
pub(super) struct CompletionDateCheck {
    /// The date's weekday is one the habit applies to, and the habit is not hidden by a
    /// low-energy day.
    pub is_scheduled: bool,
    /// The date is after today in the user's time zone.
    pub is_future: bool,
}

/// Checks a date against a habit. Returns `None` if the user has no such habit.
pub(super) async fn check_completion_date(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    habit_id: Uuid,
    date: NaiveDate,
    now: DateTime<Utc>,
) -> sqlx::Result<Option<CompletionDateCheck>> {
    sqlx::query_as!(
        CompletionDateCheck,
        r#"
        SELECT (
                   EXTRACT(ISODOW FROM $3::date)::smallint = ANY (weekdays)
                   AND (
                       low_energy
                       OR NOT EXISTS (
                           SELECT 1 FROM low_energy_days l WHERE l.user_id = $1 AND l.date = $3
                       )
                   )
               ) AS "is_scheduled!",
               $3::date > user_today($1, $4) AS "is_future!"
        FROM habits
        WHERE id = $2 AND user_id = $1
        "#,
        user_id,
        habit_id,
        date,
        now,
    )
    .fetch_optional(executor)
    .await
}

/// Records a completion. Completing the same day again changes nothing.
pub(super) async fn insert_completion(
    executor: impl PgExecutor<'_>,
    habit_id: Uuid,
    date: NaiveDate,
    now: DateTime<Utc>,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO habit_completions (habit_id, date, completed_at)
        VALUES ($1, $2, $3)
        ON CONFLICT (habit_id, date) DO NOTHING
        "#,
        habit_id,
        date,
        now,
    )
    .execute(executor)
    .await?;
    Ok(())
}

/// Removes a completion of one of the user's habits, if there is one.
pub(super) async fn delete_completion(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    habit_id: Uuid,
    date: NaiveDate,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM habit_completions c
        USING habits h
        WHERE c.habit_id = h.id AND h.id = $2 AND h.user_id = $1 AND c.date = $3
        "#,
        user_id,
        habit_id,
        date,
    )
    .execute(executor)
    .await?;
    Ok(())
}

/// Whether the user owns a habit with this ID.
pub(super) async fn habit_exists(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    habit_id: Uuid,
) -> sqlx::Result<bool> {
    sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM habits WHERE id = $2 AND user_id = $1) AS "exists!""#,
        user_id,
        habit_id,
    )
    .fetch_one(executor)
    .await
}
