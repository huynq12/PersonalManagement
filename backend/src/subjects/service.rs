//! Business rules for subjects.

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    model::{CreateSubjectRequest, Subject, UpdateSubjectRequest},
    repository::{self, SubjectChanges},
};
use crate::error::{AppError, AppResult, is_unique_violation};

/// Maps the unique violation on the name index to [`AppError::SubjectNameTaken`].
fn map_name_conflict(error: sqlx::Error) -> AppError {
    if is_unique_violation(&error) {
        AppError::SubjectNameTaken
    } else {
        AppError::Database(error)
    }
}

/// Lists the user's subjects by name, ignoring case.
pub(super) async fn list(
    db: &PgPool,
    user_id: Uuid,
    include_archived: bool,
) -> AppResult<Vec<Subject>> {
    Ok(repository::list_subjects(db, user_id, include_archived).await?)
}

/// Creates a subject.
///
/// # Errors
///
/// Returns [`AppError::SubjectNameTaken`] if the user has a subject with this name, archived
/// or not, ignoring case.
pub(super) async fn create(
    db: &PgPool,
    user_id: Uuid,
    request: CreateSubjectRequest,
) -> AppResult<Subject> {
    repository::insert_subject(db, user_id, request.name.trim(), &request.color)
        .await
        .map_err(map_name_conflict)
}

/// Updates the given fields of a subject.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if the user has no such subject, and
/// [`AppError::SubjectNameTaken`] if the new name belongs to another of their subjects.
pub(super) async fn update(
    db: &PgPool,
    user_id: Uuid,
    subject_id: Uuid,
    request: UpdateSubjectRequest,
) -> AppResult<Subject> {
    let changes = SubjectChanges {
        name: request.name.as_deref().map(str::trim),
        color: request.color.as_deref(),
        archived: request.archived,
    };

    repository::update_subject(db, user_id, subject_id, changes, Utc::now())
        .await
        .map_err(map_name_conflict)?
        .ok_or(AppError::NotFound)
}

/// Deletes a subject.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if the user has no such subject.
pub(super) async fn delete(db: &PgPool, user_id: Uuid, subject_id: Uuid) -> AppResult<()> {
    if repository::delete_subject(db, user_id, subject_id).await? {
        Ok(())
    } else {
        Err(AppError::NotFound)
    }
}
