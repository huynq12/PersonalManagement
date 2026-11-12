//! SQL for the day view. Every query filters by `user_id`.

use chrono::NaiveDate;
use sqlx::PgExecutor;
use uuid::Uuid;

use super::model::DayHabit;

/// Lists the user's active habits scheduled for the weekday of `date`, each with whether it is
/// completed on that date.
pub(super) async fn list_day_habits(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    date: NaiveDate,
) -> sqlx::Result<Vec<DayHabit>> {
    sqlx::query_as!(
        DayHabit,
        r#"
        SELECT h.id, h.name,
               EXISTS (
                   SELECT 1 FROM habit_completions c
                   WHERE c.habit_id = h.id AND c.date = $2
               ) AS "completed!"
        FROM habits h
        WHERE h.user_id = $1
          AND h.archived_at IS NULL
          AND EXTRACT(ISODOW FROM $2::date)::smallint = ANY (h.weekdays)
        ORDER BY h.sort_order, h.created_at
        "#,
        user_id,
        date,
    )
    .fetch_all(executor)
    .await
}
