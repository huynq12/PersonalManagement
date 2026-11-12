//! SQL for to-dos. Every query filters by `user_id`.

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use super::model::Task;

/// Lists a user's to-dos for one day in display order.
pub(crate) async fn list_tasks(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    date: NaiveDate,
) -> sqlx::Result<Vec<Task>> {
    sqlx::query_as!(
        Task,
        r#"
        SELECT id, date, title, is_done, done_at, sort_order, created_at
        FROM tasks
        WHERE user_id = $1 AND date = $2
        ORDER BY sort_order, created_at
        "#,
        user_id,
        date,
    )
    .fetch_all(executor)
    .await
}

/// Inserts a to-do after the user's existing ones for that day.
pub(super) async fn insert_task(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    date: NaiveDate,
    title: &str,
) -> sqlx::Result<Task> {
    sqlx::query_as!(
        Task,
        r#"
        INSERT INTO tasks (user_id, date, title, sort_order)
        VALUES (
            $1, $2, $3,
            (SELECT COALESCE(max(sort_order) + 1, 0) FROM tasks WHERE user_id = $1 AND date = $2)
        )
        RETURNING id, date, title, is_done, done_at, sort_order, created_at
        "#,
        user_id,
        date,
        title,
    )
    .fetch_one(executor)
    .await
}

/// Changes of `PATCH /tasks/{id}`; `None` keeps the current value.
pub(super) struct TaskChanges<'a> {
    pub title: Option<&'a str>,
    pub is_done: Option<bool>,
    pub date: Option<NaiveDate>,
    pub sort_order: Option<i32>,
}

/// Updates a to-do. Returns `None` if the user has no such to-do.
pub(super) async fn update_task(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    task_id: Uuid,
    changes: TaskChanges<'_>,
    now: DateTime<Utc>,
) -> sqlx::Result<Option<Task>> {
    sqlx::query_as!(
        Task,
        r#"
        UPDATE tasks
        SET title = COALESCE($3, title),
            is_done = COALESCE($4, is_done),
            done_at = CASE
                WHEN $4::boolean IS NULL THEN done_at
                -- Marking a done to-do as done again keeps the first completion time.
                WHEN $4 THEN COALESCE(done_at, $7)
                ELSE NULL
            END,
            date = COALESCE($5, date),
            sort_order = COALESCE($6, sort_order)
        WHERE id = $2 AND user_id = $1
        RETURNING id, date, title, is_done, done_at, sort_order, created_at
        "#,
        user_id,
        task_id,
        changes.title,
        changes.is_done,
        changes.date,
        changes.sort_order,
        now,
    )
    .fetch_optional(executor)
    .await
}

/// Deletes a to-do. Returns whether it existed.
pub(super) async fn delete_task(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    task_id: Uuid,
) -> sqlx::Result<bool> {
    let result = sqlx::query!(
        "DELETE FROM tasks WHERE id = $2 AND user_id = $1",
        user_id,
        task_id,
    )
    .execute(executor)
    .await?;
    Ok(result.rows_affected() > 0)
}
