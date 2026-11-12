//! SQL for subjects.
//!
//! Every query filters by `user_id`, so one user can never read or change another user's
//! subjects. Name uniqueness is enforced by the `subjects_user_id_lower_name_idx` index; the
//! service turns its violation into a domain error.

use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use super::model::Subject;

/// Lists a user's subjects by name, ignoring case.
pub(super) async fn list_subjects(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    include_archived: bool,
) -> sqlx::Result<Vec<Subject>> {
    sqlx::query_as!(
        Subject,
        r#"
        SELECT id, name, color, archived_at, created_at
        FROM subjects
        WHERE user_id = $1 AND ($2 OR archived_at IS NULL)
        ORDER BY lower(name), id
        "#,
        user_id,
        include_archived,
    )
    .fetch_all(executor)
    .await
}

/// Inserts a subject. Fails with a unique violation if the user already has the name.
pub(super) async fn insert_subject(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    name: &str,
    color: &str,
) -> sqlx::Result<Subject> {
    sqlx::query_as!(
        Subject,
        r#"
        INSERT INTO subjects (user_id, name, color)
        VALUES ($1, $2, $3)
        RETURNING id, name, color, archived_at, created_at
        "#,
        user_id,
        name,
        color,
    )
    .fetch_one(executor)
    .await
}

/// Changes of `PATCH /subjects/{id}`; `None` keeps the current value.
pub(super) struct SubjectChanges<'a> {
    pub name: Option<&'a str>,
    pub color: Option<&'a str>,
    pub archived: Option<bool>,
}

/// Updates a subject. Returns `None` if the user has no such subject. Fails with a unique
/// violation if the new name belongs to another of the user's subjects.
pub(super) async fn update_subject(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    subject_id: Uuid,
    changes: SubjectChanges<'_>,
    now: DateTime<Utc>,
) -> sqlx::Result<Option<Subject>> {
    sqlx::query_as!(
        Subject,
        r#"
        UPDATE subjects
        SET name = COALESCE($3, name),
            color = COALESCE($4, color),
            archived_at = CASE
                WHEN $5::boolean IS NULL THEN archived_at
                -- Archiving twice keeps the first archive time.
                WHEN $5 THEN COALESCE(archived_at, $6)
                ELSE NULL
            END
        WHERE id = $2 AND user_id = $1
        RETURNING id, name, color, archived_at, created_at
        "#,
        user_id,
        subject_id,
        changes.name,
        changes.color,
        changes.archived,
        now,
    )
    .fetch_optional(executor)
    .await
}

/// Deletes a subject. Returns whether it existed.
pub(super) async fn delete_subject(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    subject_id: Uuid,
) -> sqlx::Result<bool> {
    let result = sqlx::query!(
        "DELETE FROM subjects WHERE id = $2 AND user_id = $1",
        user_id,
        subject_id,
    )
    .execute(executor)
    .await?;
    Ok(result.rows_affected() > 0)
}
