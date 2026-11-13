//! SQL for study sessions.
//!
//! Every query filters by `user_id`, so one user can never read or change another user's
//! sessions. "At most one running session per user" is enforced by the
//! `study_sessions_one_running_idx` index; the service turns its violation into a domain error.

use chrono::NaiveDate;
use sqlx::PgExecutor;
use uuid::Uuid;

use super::model::StudySession;

/// Starts a running session whose length is the user's `focus_minutes` setting and whose
/// `local_date` is today in the user's time zone.
///
/// Returns `None` if `subject_id` is not one of the user's subjects. Fails with a unique
/// violation if the user already has a running session.
pub(super) async fn insert_session(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    subject_id: Option<Uuid>,
) -> sqlx::Result<Option<StudySession>> {
    sqlx::query_as!(
        StudySession,
        r#"
        INSERT INTO study_sessions (user_id, subject_id, status, planned_minutes, local_date)
        SELECT u.id, $2, 'running', s.focus_minutes, (now() AT TIME ZONE u.timezone)::date
        FROM users u
        JOIN user_settings s ON s.user_id = u.id
        WHERE u.id = $1
          AND ($2::uuid IS NULL
               OR EXISTS (SELECT 1 FROM subjects WHERE id = $2 AND user_id = $1))
        RETURNING id, subject_id, status, planned_minutes, started_at, ended_at,
                  duration_seconds, local_date
        "#,
        user_id,
        subject_id,
    )
    .fetch_optional(executor)
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

/// Completes a running session, recording the elapsed time capped at the planned length.
/// Returns `None` if the user has no such running session.
pub(super) async fn complete_session(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    session_id: Uuid,
) -> sqlx::Result<Option<StudySession>> {
    sqlx::query_as!(
        StudySession,
        r#"
        UPDATE study_sessions
        SET status = 'completed',
            ended_at = now(),
            duration_seconds = LEAST(
                GREATEST(EXTRACT(EPOCH FROM now() - started_at)::int, 0),
                planned_minutes * 60
            )
        WHERE id = $2 AND user_id = $1 AND status = 'running'
        RETURNING id, subject_id, status, planned_minutes, started_at, ended_at,
                  duration_seconds, local_date
        "#,
        user_id,
        session_id,
    )
    .fetch_optional(executor)
    .await
}

/// Cancels a running session. Returns `None` if the user has no such running session.
pub(super) async fn cancel_session(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    session_id: Uuid,
) -> sqlx::Result<Option<StudySession>> {
    sqlx::query_as!(
        StudySession,
        r#"
        UPDATE study_sessions
        SET status = 'cancelled', ended_at = now()
        WHERE id = $2 AND user_id = $1 AND status = 'running'
        RETURNING id, subject_id, status, planned_minutes, started_at, ended_at,
                  duration_seconds, local_date
        "#,
        user_id,
        session_id,
    )
    .fetch_optional(executor)
    .await
}

/// Whether the user has a session with this ID, in any status.
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

/// Lists the user's sessions that started between two local dates, both included, newest first.
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
