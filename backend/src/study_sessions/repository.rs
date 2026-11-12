//! SQL for study sessions.
//!
//! Every query filters by `user_id`, so one user can never read or change another user's
//! sessions. The `study_sessions_one_running_idx` index allows one running session per user; the
//! service turns its violation into a domain error.

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use super::model::StudySession;

/// Inserts a running session that starts at `now`. `planned_minutes` defaults to the user's
/// `focus_minutes` setting. Fails with a unique violation if the user already has a running
/// session.
pub(super) async fn insert_session(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    subject_id: Option<Uuid>,
    planned_minutes: Option<i16>,
    now: DateTime<Utc>,
) -> sqlx::Result<StudySession> {
    sqlx::query_as!(
        StudySession,
        r#"
        INSERT INTO study_sessions (user_id, subject_id, status, planned_minutes, started_at, local_date)
        VALUES (
            $1, $2, 'running',
            COALESCE($3, (SELECT focus_minutes FROM user_settings WHERE user_id = $1)),
            $4, user_today($1, $4)
        )
        RETURNING id, subject_id, status, planned_minutes, started_at, ended_at,
                  duration_seconds, local_date
        "#,
        user_id,
        subject_id,
        planned_minutes,
        now,
    )
    .fetch_one(executor)
    .await
}

/// The user's running session, if any.
pub(super) async fn find_running(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
) -> sqlx::Result<Option<StudySession>> {
    sqlx::query_as!(
        StudySession,
        r#"
        SELECT id, subject_id, status, planned_minutes, started_at, ended_at,
               duration_seconds, local_date
        FROM study_sessions
        WHERE user_id = $1 AND status = 'running'
        "#,
        user_id,
    )
    .fetch_optional(executor)
    .await
}

/// Ends a running session as `completed` or `cancelled`. A completed session gets the time
/// elapsed since it started, capped at its planned length. Returns `None` if the user has no
/// such running session.
pub(super) async fn finish_session(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    session_id: Uuid,
    completed: bool,
    now: DateTime<Utc>,
) -> sqlx::Result<Option<StudySession>> {
    sqlx::query_as!(
        StudySession,
        r#"
        UPDATE study_sessions
        SET status = CASE WHEN $3 THEN 'completed' ELSE 'cancelled' END,
            ended_at = $4,
            duration_seconds = CASE WHEN $3 THEN
                LEAST(
                    GREATEST(floor(EXTRACT(EPOCH FROM ($4 - started_at)))::integer, 0),
                    planned_minutes * 60
                )
            END
        WHERE id = $2 AND user_id = $1 AND status = 'running'
        RETURNING id, subject_id, status, planned_minutes, started_at, ended_at,
                  duration_seconds, local_date
        "#,
        user_id,
        session_id,
        completed,
        now,
    )
    .fetch_optional(executor)
    .await
}

/// Whether the user has a session with this ID, in any state.
pub(super) async fn session_exists(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    session_id: Uuid,
) -> sqlx::Result<bool> {
    let row = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM study_sessions WHERE id = $2 AND user_id = $1) AS "exists!""#,
        user_id,
        session_id,
    )
    .fetch_one(executor)
    .await?;
    Ok(row)
}

/// Lists the sessions started on local dates from `from` to `to`, newest first.
pub(super) async fn list_sessions(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    from: NaiveDate,
    to: NaiveDate,
) -> sqlx::Result<Vec<StudySession>> {
    sqlx::query_as!(
        StudySession,
        r#"
        SELECT id, subject_id, status, planned_minutes, started_at, ended_at,
               duration_seconds, local_date
        FROM study_sessions
        WHERE user_id = $1 AND local_date BETWEEN $2 AND $3
        ORDER BY started_at DESC, id
        "#,
        user_id,
        from,
        to,
    )
    .fetch_all(executor)
    .await
}

/// Deletes a session. Returns whether it existed.
pub(super) async fn delete_session(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    session_id: Uuid,
) -> sqlx::Result<bool> {
    let result = sqlx::query!(
        "DELETE FROM study_sessions WHERE id = $2 AND user_id = $1",
        user_id,
        session_id,
    )
    .execute(executor)
    .await?;
    Ok(result.rows_affected() > 0)
}
