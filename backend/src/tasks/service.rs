//! Business rules for to-dos.

use chrono::{NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    model::{CreateTaskRequest, Task, UpdateTaskRequest},
    repository::{self, TaskChanges},
};
use crate::error::{AppError, AppResult};

/// Lists the user's to-dos for one day.
pub(super) async fn list(db: &PgPool, user_id: Uuid, date: NaiveDate) -> AppResult<Vec<Task>> {
    Ok(repository::list_tasks(db, user_id, date).await?)
}

/// Creates a to-do at the end of its day's list.
pub(super) async fn create(
    db: &PgPool,
    user_id: Uuid,
    request: CreateTaskRequest,
) -> AppResult<Task> {
    Ok(repository::insert_task(db, user_id, request.date, request.title.trim()).await?)
}

/// Updates the given fields of a to-do.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if the user has no such to-do.
pub(super) async fn update(
    db: &PgPool,
    user_id: Uuid,
    task_id: Uuid,
    request: UpdateTaskRequest,
) -> AppResult<Task> {
    let changes = TaskChanges {
        title: request.title.as_deref().map(str::trim),
        is_done: request.is_done,
        date: request.date,
        sort_order: request.sort_order,
    };

    repository::update_task(db, user_id, task_id, changes, Utc::now())
        .await?
        .ok_or(AppError::NotFound)
}

/// Deletes a to-do.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if the user has no such to-do.
pub(super) async fn delete(db: &PgPool, user_id: Uuid, task_id: Uuid) -> AppResult<()> {
    if repository::delete_task(db, user_id, task_id).await? {
        Ok(())
    } else {
        Err(AppError::NotFound)
    }
}
