//! Business rules for the Pomodoro settings.

use anyhow::anyhow;
use sqlx::PgPool;
use uuid::Uuid;

use super::{model::Settings, repository};
use crate::error::{AppError, AppResult};

/// A user always has a settings row, created together with the account; its absence is a bug.
fn missing_row() -> AppError {
    AppError::Internal(anyhow!("user settings row is missing"))
}

/// The user's settings.
pub(super) async fn get(db: &PgPool, user_id: Uuid) -> AppResult<Settings> {
    repository::find_settings(db, user_id)
        .await?
        .ok_or_else(missing_row)
}

/// Replaces the user's settings.
pub(super) async fn update(db: &PgPool, user_id: Uuid, settings: Settings) -> AppResult<Settings> {
    repository::update_settings(db, user_id, &settings)
        .await?
        .ok_or_else(missing_row)
}
