//! SQL for the settings feature.

use sqlx::PgExecutor;
use uuid::Uuid;

use super::model::PomodoroSettings;

/// Reads the user's settings. Every user has a row from registration, so a missing row is a
/// bug and surfaces as a database error.
pub(super) async fn get_settings(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
) -> sqlx::Result<PomodoroSettings> {
    sqlx::query_as!(
        PomodoroSettings,
        r#"
        SELECT focus_minutes, short_break_minutes, long_break_minutes, long_break_every
        FROM user_settings
        WHERE user_id = $1
        "#,
        user_id,
    )
    .fetch_one(executor)
    .await
}

/// Replaces every setting of the user and returns the stored values.
pub(super) async fn replace_settings(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    settings: &PomodoroSettings,
) -> sqlx::Result<PomodoroSettings> {
    sqlx::query_as!(
        PomodoroSettings,
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
    .fetch_one(executor)
    .await
}
