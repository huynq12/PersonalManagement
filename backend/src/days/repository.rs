//! SQL for the day view and day modes. Every query filters by `user_id`.

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use super::model::{DayHabit, DayModeState, DayWork};

/// Lists the user's active habits scheduled for the weekday of `date`, each with whether it is
/// completed on that date.
///
/// On a low-energy day only habits kept for such days are listed, under their easier name when
/// they have one.
pub(super) async fn list_day_habits(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    date: NaiveDate,
) -> sqlx::Result<Vec<DayHabit>> {
    sqlx::query_as!(
        DayHabit,
        r#"
        SELECT h.id,
               CASE WHEN mode.is_low_energy THEN COALESCE(h.low_energy_name, h.name)
                    ELSE h.name
               END AS "name!",
               EXISTS (
                   SELECT 1 FROM habit_completions c
                   WHERE c.habit_id = h.id AND c.date = $2
               ) AS "completed!"
        FROM habits h
        CROSS JOIN (
            SELECT EXISTS (
                SELECT 1 FROM low_energy_days d WHERE d.user_id = $1 AND d.date = $2
            ) AS is_low_energy
        ) AS mode
        WHERE h.user_id = $1
          AND h.archived_at IS NULL
          AND EXTRACT(ISODOW FROM $2::date)::smallint = ANY (h.weekdays)
          AND (NOT mode.is_low_energy OR h.low_energy)
        ORDER BY h.sort_order, h.created_at
        "#,
        user_id,
        date,
    )
    .fetch_all(executor)
    .await
}

/// The mode of `date` and whether it can be changed. `now` decides which date is today.
///
/// Here and in the two writes below, only completions of active habits lock the mode: an
/// archived habit is not on the Today screen, so the user could not uncheck it to unlock.
pub(super) async fn day_mode_state(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    date: NaiveDate,
    now: DateTime<Utc>,
) -> sqlx::Result<DayModeState> {
    sqlx::query_as!(
        DayModeState,
        r#"
        SELECT EXISTS (
                   SELECT 1 FROM low_energy_days d WHERE d.user_id = $1 AND d.date = $2
               ) AS "is_low_energy!",
               (
                   $2::date IS DISTINCT FROM user_today($1, $3)
                   OR EXISTS (
                       SELECT 1 FROM habit_completions c
                       JOIN habits h ON h.id = c.habit_id
                       WHERE h.user_id = $1 AND h.archived_at IS NULL AND c.date = $2
                   )
               ) AS "locked!"
        "#,
        user_id,
        date,
        now,
    )
    .fetch_one(executor)
    .await
}

/// The calendar date it is for the user at `now`.
pub(super) async fn user_today(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    now: DateTime<Utc>,
) -> sqlx::Result<NaiveDate> {
    sqlx::query_scalar!(r#"SELECT user_today($1, $2) AS "today!""#, user_id, now)
        .fetch_one(executor)
        .await
}

/// Marks `date` as a low-energy day unless a habit is checked on it. Returns whether the day
/// was newly marked. The check and the write are one statement, so a habit checked at the same
/// moment cannot slip in between.
pub(super) async fn mark_low_energy(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    date: NaiveDate,
) -> sqlx::Result<bool> {
    let result = sqlx::query!(
        r#"
        INSERT INTO low_energy_days (user_id, date)
        SELECT $1, $2::date
        WHERE NOT EXISTS (
            SELECT 1 FROM habit_completions c
            JOIN habits h ON h.id = c.habit_id
            WHERE h.user_id = $1 AND h.archived_at IS NULL AND c.date = $2
        )
        ON CONFLICT (user_id, date) DO NOTHING
        "#,
        user_id,
        date,
    )
    .execute(executor)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Makes `date` a normal day again unless a habit is checked on it. Returns whether the mark
/// was removed.
pub(super) async fn unmark_low_energy(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    date: NaiveDate,
) -> sqlx::Result<bool> {
    let result = sqlx::query!(
        r#"
        DELETE FROM low_energy_days
        WHERE user_id = $1 AND date = $2
          AND NOT EXISTS (
              SELECT 1 FROM habit_completions c
              JOIN habits h ON h.id = c.habit_id
              WHERE h.user_id = $1 AND h.archived_at IS NULL AND c.date = $2
          )
        "#,
        user_id,
        date,
    )
    .execute(executor)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// The user's work entry for `date`, if any.
pub(super) async fn day_work(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    date: NaiveDate,
) -> sqlx::Result<Option<DayWork>> {
    let row = sqlx::query!(
        r#"
        SELECT (hours * 100)::int4 AS "hours_hundredths!",
               round(hours * hourly_wage)::int8 AS "pay!"
        FROM work_entries
        WHERE user_id = $1 AND date = $2
        "#,
        user_id,
        date,
    )
    .fetch_optional(executor)
    .await?;
    Ok(row.map(|row| DayWork {
        hours: f64::from(row.hours_hundredths) / 100.0,
        pay: row.pay,
    }))
}

/// The total of the user's expense transactions dated `date`; 0 when there are none.
pub(super) async fn day_expense_total(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    date: NaiveDate,
) -> sqlx::Result<i64> {
    sqlx::query_scalar!(
        r#"
        SELECT COALESCE(sum(t.amount), 0)::int8 AS "total!"
        FROM transactions t
        JOIN finance_categories c ON c.id = t.category_id
        WHERE t.user_id = $1 AND t.date = $2 AND c.kind = 'expense'
        "#,
        user_id,
        date,
    )
    .fetch_one(executor)
    .await
}
