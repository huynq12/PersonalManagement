//! Assembles the day view from the habits and to-dos features, and changes the day mode.

use chrono::{NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    model::{DayMode, DayView},
    repository,
};
use crate::{
    error::{AppError, AppResult},
    tasks,
    validation::field_error,
};

/// Loads the mode, the scheduled habits, the to-dos, and the money figures of one date.
pub(super) async fn day_view(db: &PgPool, user_id: Uuid, date: NaiveDate) -> AppResult<DayView> {
    // The queries are independent, so run them concurrently.
    let (mode, habits, tasks, work, expense_total) = tokio::try_join!(
        repository::day_mode_state(db, user_id, date, Utc::now()),
        repository::list_day_habits(db, user_id, date),
        tasks::repository::list_tasks(db, user_id, date),
        repository::day_work(db, user_id, date),
        repository::day_expense_total(db, user_id, date),
    )?;

    Ok(DayView {
        date,
        mode: DayMode::from_low_energy(mode.is_low_energy),
        mode_locked: mode.locked,
        habits,
        tasks,
        work,
        expense_total,
    })
}

/// Sets the mode of `date`. Setting the mode a day already has is not an error.
///
/// # Errors
///
/// Returns [`AppError::Validation`] on the `date` field with code `not_today` unless `date` is
/// today in the user's time zone, and [`AppError::DayModeLocked`] if the mode would change while
/// a habit is checked for today.
pub(super) async fn set_mode(
    db: &PgPool,
    user_id: Uuid,
    date: NaiveDate,
    mode: DayMode,
) -> AppResult<()> {
    let now = Utc::now();
    if date != repository::user_today(db, user_id, now).await? {
        return Err(field_error("date", "not_today"));
    }

    let wanted_low_energy = mode == DayMode::LowEnergy;
    let current = repository::day_mode_state(db, user_id, date, now).await?;
    if current.is_low_energy == wanted_low_energy {
        return Ok(());
    }

    let changed = if wanted_low_energy {
        repository::mark_low_energy(db, user_id, date).await?
    } else {
        repository::unmark_low_energy(db, user_id, date).await?
    };
    if changed {
        return Ok(());
    }

    // Nothing changed: either a habit is checked, or a concurrent request already set the mode.
    let after = repository::day_mode_state(db, user_id, date, now).await?;
    if after.is_low_energy == wanted_low_energy {
        Ok(())
    } else {
        Err(AppError::DayModeLocked)
    }
}
