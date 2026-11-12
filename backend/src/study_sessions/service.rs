//! Business rules for study sessions.

use chrono::{NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    model::{StartSessionRequest, StudySession},
    repository,
};
use crate::{
    error::{AppError, AppResult, is_unique_violation},
    subjects,
    validation::field_error,
};

/// Starts a focus session.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if the subject is not one of the user's, a validation error
/// if it is archived, and [`AppError::StudySessionAlreadyRunning`] if the user already has a
/// running session.
pub(super) async fn start(
    db: &PgPool,
    user_id: Uuid,
    request: StartSessionRequest,
) -> AppResult<StudySession> {
    if let Some(subject_id) = request.subject_id {
        let subject = subjects::repository::find_subject(db, user_id, subject_id)
            .await?
            .ok_or(AppError::NotFound)?;
        if subject.archived_at.is_some() {
            return Err(field_error("subject_id", "archived"));
        }
    }

    repository::insert_session(
        db,
        user_id,
        request.subject_id,
        request.planned_minutes,
        Utc::now(),
    )
    .await
    .map_err(|error| {
        if is_unique_violation(&error) {
            AppError::StudySessionAlreadyRunning
        } else {
            AppError::Database(error)
        }
    })
}

/// The user's running session, if any.
pub(super) async fn current(db: &PgPool, user_id: Uuid) -> AppResult<Option<StudySession>> {
    Ok(repository::find_running(db, user_id).await?)
}

/// Completes (`completed = true`) or cancels a running session.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if the user has no such session, and
/// [`AppError::StudySessionNotRunning`] if it already ended.
pub(super) async fn finish(
    db: &PgPool,
    user_id: Uuid,
    session_id: Uuid,
    completed: bool,
) -> AppResult<StudySession> {
    if let Some(session) =
        repository::finish_session(db, user_id, session_id, completed, Utc::now()).await?
    {
        return Ok(session);
    }

    // Nothing was updated: either the session is not the user's, or it is no longer running.
    if repository::session_exists(db, user_id, session_id).await? {
        Err(AppError::StudySessionNotRunning)
    } else {
        Err(AppError::NotFound)
    }
}

/// Lists the sessions started between two local dates, both included.
///
/// # Errors
///
/// Returns a validation error if `to` is before `from`.
pub(super) async fn list(
    db: &PgPool,
    user_id: Uuid,
    from: NaiveDate,
    to: NaiveDate,
) -> AppResult<Vec<StudySession>> {
    if to < from {
        return Err(field_error("to", "before_from"));
    }
    Ok(repository::list_sessions(db, user_id, from, to).await?)
}

/// Deletes a session.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if the user has no such session.
pub(super) async fn delete(db: &PgPool, user_id: Uuid, session_id: Uuid) -> AppResult<()> {
    if repository::delete_session(db, user_id, session_id).await? {
        Ok(())
    } else {
        Err(AppError::NotFound)
    }
}
