//! Business rules for study sessions.

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use super::{model::StudySession, repository};
use crate::error::{AppError, AppResult, FieldErrors, is_unique_violation};

/// Starts a focus session of the user's configured length.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if `subject_id` is not one of the user's subjects, and
/// [`AppError::StudySessionAlreadyRunning`] if a session is already running.
pub(super) async fn start(
    db: &PgPool,
    user_id: Uuid,
    subject_id: Option<Uuid>,
) -> AppResult<StudySession> {
    repository::insert_session(db, user_id, subject_id)
        .await
        .map_err(|error| {
            if is_unique_violation(&error) {
                AppError::StudySessionAlreadyRunning
            } else {
                AppError::Database(error)
            }
        })?
        .ok_or(AppError::NotFound)
}

/// The running session, if any.
pub(super) async fn current(db: &PgPool, user_id: Uuid) -> AppResult<Option<StudySession>> {
    Ok(repository::find_running(db, user_id).await?)
}

/// Completes the session; the server computes the duration.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if the user has no such session, and
/// [`AppError::StudySessionNotRunning`] if it already ended.
pub(super) async fn complete(
    db: &PgPool,
    user_id: Uuid,
    session_id: Uuid,
) -> AppResult<StudySession> {
    match repository::complete_session(db, user_id, session_id).await? {
        Some(session) => Ok(session),
        None => Err(not_updated(db, user_id, session_id).await?),
    }
}

/// Cancels the session.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if the user has no such session, and
/// [`AppError::StudySessionNotRunning`] if it already ended.
pub(super) async fn cancel(
    db: &PgPool,
    user_id: Uuid,
    session_id: Uuid,
) -> AppResult<StudySession> {
    match repository::cancel_session(db, user_id, session_id).await? {
        Some(session) => Ok(session),
        None => Err(not_updated(db, user_id, session_id).await?),
    }
}

/// Explains why a session could not be completed or cancelled: it does not exist for this
/// user, or it is no longer running.
async fn not_updated(db: &PgPool, user_id: Uuid, session_id: Uuid) -> AppResult<AppError> {
    if repository::session_exists(db, user_id, session_id).await? {
        Ok(AppError::StudySessionNotRunning)
    } else {
        Ok(AppError::NotFound)
    }
}

/// Lists sessions that started between two local dates, both included.
///
/// # Errors
///
/// Returns [`AppError::Validation`] if `from` is after `to`.
pub(super) async fn list(
    db: &PgPool,
    user_id: Uuid,
    from: NaiveDate,
    to: NaiveDate,
) -> AppResult<Vec<StudySession>> {
    if from > to {
        let mut fields = FieldErrors::new();
        fields.insert("to".to_owned(), vec!["range".to_owned()]);
        return Err(AppError::Validation(fields));
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
