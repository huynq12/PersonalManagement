//! SQL for settings. Every query filters by `user_id`.

use sqlx::PgExecutor;
use uuid::Uuid;

use super::model::SettingsRow;

/// Creates the default settings row for a user who has none; does nothing otherwise.
///
/// New users get their row at sign-up; this only covers accounts that somehow lack one.
pub(super) async fn ensure_row(executor: impl PgExecutor<'_>, user_id: Uuid) -> sqlx::Result<()> {
    sqlx::query!(
        "INSERT INTO user_settings (user_id) VALUES ($1) ON CONFLICT (user_id) DO NOTHING",
        user_id,
    )
    .execute(executor)
    .await?;
    Ok(())
}

/// The user's settings row, or `None` if the user has none.
pub(super) async fn find(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
) -> sqlx::Result<Option<SettingsRow>> {
    sqlx::query_as!(
        SettingsRow,
        r#"
        SELECT focus_minutes, short_break_minutes, long_break_minutes, long_break_every,
               hourly_wage, (base_work_hours * 100)::int4 AS base_work_hundredths,
               monthly_budget
        FROM user_settings
        WHERE user_id = $1
        "#,
        user_id,
    )
    .fetch_optional(executor)
    .await
}

/// Changes of `PATCH /settings`.
///
/// `None` keeps the current value. For the nullable settings the outer `Option` says whether to
/// change the column and the inner one is the new value, where `None` clears it.
pub(super) struct SettingsChanges {
    pub focus_minutes: Option<i16>,
    pub short_break_minutes: Option<i16>,
    pub long_break_minutes: Option<i16>,
    pub long_break_every: Option<i16>,
    pub hourly_wage: Option<Option<i64>>,
    pub base_work_hundredths: Option<Option<i32>>,
    pub monthly_budget: Option<Option<i64>>,
}

/// Applies the changes in one statement and returns the updated row, or `None` if the user
/// has no settings row.
pub(super) async fn update(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    changes: SettingsChanges,
) -> sqlx::Result<Option<SettingsRow>> {
    sqlx::query_as!(
        SettingsRow,
        r#"
        UPDATE user_settings
        SET focus_minutes = COALESCE($2, focus_minutes),
            short_break_minutes = COALESCE($3, short_break_minutes),
            long_break_minutes = COALESCE($4, long_break_minutes),
            long_break_every = COALESCE($5, long_break_every),
            hourly_wage = CASE WHEN $6 THEN $7 ELSE hourly_wage END,
            base_work_hours = CASE WHEN $8 THEN $9::int4::numeric / 100 ELSE base_work_hours END,
            monthly_budget = CASE WHEN $10 THEN $11 ELSE monthly_budget END
        WHERE user_id = $1
        RETURNING focus_minutes, short_break_minutes, long_break_minutes, long_break_every,
                  hourly_wage, (base_work_hours * 100)::int4 AS base_work_hundredths,
                  monthly_budget
        "#,
        user_id,
        changes.focus_minutes,
        changes.short_break_minutes,
        changes.long_break_minutes,
        changes.long_break_every,
        changes.hourly_wage.is_some(),
        changes.hourly_wage.flatten(),
        changes.base_work_hundredths.is_some(),
        changes.base_work_hundredths.flatten(),
        changes.monthly_budget.is_some(),
        changes.monthly_budget.flatten(),
    )
    .fetch_optional(executor)
    .await
}
