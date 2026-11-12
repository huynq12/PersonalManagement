//! SQL for the Pomodoro settings. Every query filters by `user_id`.

use sqlx::PgExecutor;
use uuid::Uuid;

use super::model::Settings;

/// The user's settings. `None` only if the row is missing, which registration prevents.
pub(super) async fn find_settings(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
) -> sqlx::Result<Option<Settings>> {
    sqlx::query_as!(
        Settings,
        r#"
        SELECT focus_minutes, short_break_minutes, long_break_minutes, long_break_every
        FROM user_settings
        WHERE user_id = $1
        "#,
        user_id,
    )
    .fetch_optional(executor)
    .await
}

/// Replaces the user's settings. Returns `None` if the row is missing.
pub(super) async fn update_settings(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    settings: &Settings,
) -> sqlx::Result<Option<Settings>> {
    sqlx::query_as!(
        Settings,
        r#"
        UPDATE user_settings
        SET focus_minutes = $2,
            short_break_minutes = $3,
            long_break_minutes = $4,
            long_break_every = $5
        WHERE user_id = $1
        RETURNING focus_minutes, short_break_minutes, long_break_minutes, long_break_every
        "#,
        user_id,
        settings.focus_minutes,
        settings.short_break_minutes,
        settings.long_break_minutes,
        settings.long_break_every,
    )
    .fetch_optional(executor)
    .await
}
