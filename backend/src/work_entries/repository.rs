//! SQL for the work log. Every query filters by `user_id`.

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use super::model::WorkEntryRow;

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

/// The entries of the month starting at `month_start`, oldest day first.
pub(super) async fn list_month(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    month_start: NaiveDate,
) -> sqlx::Result<Vec<WorkEntryRow>> {
    sqlx::query_as!(
        WorkEntryRow,
        r#"
        SELECT date, (hours * 100)::int4 AS "hours_hundredths!", hourly_wage,
               (base_hours * 100)::int4 AS "base_hundredths!",
               round(hours * hourly_wage)::int8 AS "pay!", note
        FROM work_entries
        WHERE user_id = $1 AND date >= $2 AND date < ($2::date + interval '1 month')::date
        ORDER BY date
        "#,
        user_id,
        month_start,
    )
    .fetch_all(executor)
    .await
}

/// Changes the hours and note of an existing entry, keeping its copied wage and base hours.
/// Returns `None` if the day has no entry.
pub(super) async fn update(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    date: NaiveDate,
    hours_hundredths: i32,
    note: Option<&str>,
) -> sqlx::Result<Option<WorkEntryRow>> {
    sqlx::query_as!(
        WorkEntryRow,
        r#"
        UPDATE work_entries
        SET hours = $3::int4::numeric / 100, note = $4, updated_at = now()
        WHERE user_id = $1 AND date = $2
        RETURNING date, (hours * 100)::int4 AS "hours_hundredths!", hourly_wage,
                  (base_hours * 100)::int4 AS "base_hundredths!",
                  round(hours * hourly_wage)::int8 AS "pay!", note
        "#,
        user_id,
        date,
        hours_hundredths,
        note,
    )
    .fetch_optional(executor)
    .await
}

/// Creates the day's entry with the wage and base hours currently in the user's settings.
///
/// Returns `None` if either setting is not set. If a concurrent request created the entry
/// first, this updates its hours and note instead, so the day never gets two rows.
pub(super) async fn insert_from_settings(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    date: NaiveDate,
    hours_hundredths: i32,
    note: Option<&str>,
) -> sqlx::Result<Option<WorkEntryRow>> {
    sqlx::query_as!(
        WorkEntryRow,
        r#"
        INSERT INTO work_entries (user_id, date, hours, hourly_wage, base_hours, note)
        SELECT $1, $2::date, $3::int4::numeric / 100, s.hourly_wage, s.base_work_hours, $4
        FROM user_settings s
        WHERE s.user_id = $1 AND s.hourly_wage IS NOT NULL AND s.base_work_hours IS NOT NULL
        ON CONFLICT (user_id, date)
        DO UPDATE SET hours = EXCLUDED.hours, note = EXCLUDED.note, updated_at = now()
        RETURNING date, (hours * 100)::int4 AS "hours_hundredths!", hourly_wage,
                  (base_hours * 100)::int4 AS "base_hundredths!",
                  round(hours * hourly_wage)::int8 AS "pay!", note
        "#,
        user_id,
        date,
        hours_hundredths,
        note,
    )
    .fetch_optional(executor)
    .await
}

/// Deletes the day's entry. Returns whether one existed.
pub(super) async fn delete(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    date: NaiveDate,
) -> sqlx::Result<bool> {
    let result = sqlx::query!(
        "DELETE FROM work_entries WHERE user_id = $1 AND date = $2",
        user_id,
        date,
    )
    .execute(executor)
    .await?;
    Ok(result.rows_affected() > 0)
}
