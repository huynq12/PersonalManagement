//! SQL for the weekly statistics. Every query filters by `user_id`.

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use super::model::{DayRow, HabitStreaks, SubjectStats};

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

/// One row per date from `from` to `to`, both included.
///
/// Habits count on a date only when `habit_scheduled_on` says they are scheduled, the same rule
/// that decides the Today screen and the streaks. Active habits only; to-dos always count.
pub(super) async fn list_days(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    from: NaiveDate,
    to: NaiveDate,
) -> sqlx::Result<Vec<DayRow>> {
    sqlx::query_as!(
        DayRow,
        r#"
        SELECT day::date AS "date!",
               EXISTS (
                   SELECT 1 FROM low_energy_days l WHERE l.user_id = $1 AND l.date = day::date
               ) AS "is_low_energy!",
               COALESCE((
                   SELECT sum(s.duration_seconds) FROM study_sessions s
                   WHERE s.user_id = $1 AND s.local_date = day::date AND s.status = 'completed'
               ), 0)::int8 AS "study_seconds!",
               (
                   SELECT count(*) FROM study_sessions s
                   WHERE s.user_id = $1 AND s.local_date = day::date AND s.status = 'completed'
               ) AS "pomodoros!",
               (
                   SELECT count(*) FROM habits h
                   WHERE h.user_id = $1 AND h.archived_at IS NULL
                     AND habit_scheduled_on($1, h.weekdays, h.low_energy, day::date)
               ) AS "habits_scheduled!",
               (
                   SELECT count(*) FROM habit_completions c
                   JOIN habits h ON h.id = c.habit_id
                   WHERE h.user_id = $1 AND h.archived_at IS NULL AND c.date = day::date
                     AND habit_scheduled_on($1, h.weekdays, h.low_energy, day::date)
               ) AS "habits_done!",
               (
                   SELECT count(*) FROM tasks t WHERE t.user_id = $1 AND t.date = day::date
               ) AS "tasks_total!",
               (
                   SELECT count(*) FROM tasks t
                   WHERE t.user_id = $1 AND t.date = day::date AND t.is_done
               ) AS "tasks_done!"
        FROM generate_series($2::date::timestamp, $3::date::timestamp, interval '1 day') AS day
        ORDER BY day
        "#,
        user_id,
        from,
        to,
    )
    .fetch_all(executor)
    .await
}

/// Study time of the completed sessions that started from `from` to `to`, grouped by subject,
/// longest first. Archived subjects are included; sessions without a subject form one group.
pub(super) async fn list_subjects(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    from: NaiveDate,
    to: NaiveDate,
) -> sqlx::Result<Vec<SubjectStats>> {
    sqlx::query_as!(
        SubjectStats,
        r#"
        SELECT s.subject_id AS "subject_id?",
               sub.name AS "name?",
               sub.color AS "color?",
               COALESCE(sum(s.duration_seconds), 0)::int8 AS "study_seconds!",
               count(*) AS "pomodoros!"
        FROM study_sessions s
        LEFT JOIN subjects sub ON sub.id = s.subject_id
        WHERE s.user_id = $1 AND s.status = 'completed' AND s.local_date BETWEEN $2 AND $3
        GROUP BY s.subject_id, sub.name, sub.color
        ORDER BY COALESCE(sum(s.duration_seconds), 0) DESC, sub.name
        "#,
        user_id,
        from,
        to,
    )
    .fetch_all(executor)
    .await
}

/// The current and the longest streak of every active habit, with "today" being `today`.
pub(super) async fn list_habit_streaks(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    today: NaiveDate,
) -> sqlx::Result<Vec<HabitStreaks>> {
    sqlx::query_as!(
        HabitStreaks,
        r#"
        SELECT id,
               name,
               habit_current_streak(id, user_id, weekdays, low_energy, $2) AS "current_streak!",
               habit_longest_streak(id, user_id, weekdays, low_energy, $2) AS "longest_streak!"
        FROM habits
        WHERE user_id = $1 AND archived_at IS NULL
        ORDER BY sort_order, created_at
        "#,
        user_id,
        today,
    )
    .fetch_all(executor)
    .await
}
