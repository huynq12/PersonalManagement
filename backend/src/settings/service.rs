//! Business rules for settings.

use sqlx::PgPool;
use uuid::Uuid;

use super::{
    model::{Settings, UpdateSettingsRequest},
    repository::{self, SettingsChanges},
};
use crate::{
    error::{AppError, AppResult},
    validation::{field_error, to_hundredths},
};

/// The user's settings. Creates the default row first if the user has none.
pub(super) async fn get(db: &PgPool, user_id: Uuid) -> AppResult<Settings> {
    if let Some(row) = repository::find(db, user_id).await? {
        return Ok(row.into());
    }
    repository::ensure_row(db, user_id).await?;
    repository::find(db, user_id)
        .await?
        .map(Settings::from)
        .ok_or(AppError::Unauthorized)
}

/// Applies the fields present in the request and returns the new settings.
///
/// # Errors
///
/// Returns [`AppError::Unauthorized`] if the user no longer exists.
pub(super) async fn update(
    db: &PgPool,
    user_id: Uuid,
    request: UpdateSettingsRequest,
) -> AppResult<Settings> {
    // The request was validated, so the conversion only fails if that validation is bypassed.
    let base_work_hundredths = request
        .base_work_hours
        .map(|hours| {
            hours
                .map(|hours| {
                    to_hundredths(hours).ok_or_else(|| field_error("base_work_hours", "range"))
                })
                .transpose()
        })
        .transpose()?;

    let changes = SettingsChanges {
        focus_minutes: request.focus_minutes,
        short_break_minutes: request.short_break_minutes,
        long_break_minutes: request.long_break_minutes,
        long_break_every: request.long_break_every,
        hourly_wage: request.hourly_wage,
        base_work_hundredths,
        monthly_budget: request.monthly_budget,
    };

    repository::ensure_row(db, user_id).await?;
    repository::update(db, user_id, changes)
        .await?
        .map(Settings::from)
        .ok_or(AppError::Unauthorized)
}
