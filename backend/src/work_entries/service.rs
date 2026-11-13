//! Business rules for the work log.

use chrono::{NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    model::{MonthView, PutWorkEntryRequest, Summary, WorkEntry, parse_month},
    repository,
};
use crate::{
    error::{AppError, AppResult},
    validation::{field_error, to_hundredths},
};

/// The month's entries and totals.
///
/// # Errors
///
/// Returns [`AppError::Validation`] on the `month` field with code `format` if `month` is not
/// `YYYY-MM`.
pub(super) async fn list(db: &PgPool, user_id: Uuid, month: &str) -> AppResult<MonthView> {
    let month_start = parse_month(month).ok_or_else(|| field_error("month", "format"))?;
    let rows = repository::list_month(db, user_id, month_start).await?;
    let summary = Summary::of(&rows);
    Ok(MonthView {
        entries: rows.into_iter().map(WorkEntry::from).collect(),
        summary,
    })
}

/// Creates or updates the entry of `date`.
///
/// A new entry copies the wage and base hours from the settings; an existing one keeps the
/// values it already has.
///
/// # Errors
///
/// Returns [`AppError::Validation`] on the `date` field with code `future` for a date after
/// today in the user's time zone, and [`AppError::WorkSettingsMissing`] if a new entry needs a
/// wage or base hours that are not set.
pub(super) async fn put(
    db: &PgPool,
    user_id: Uuid,
    date: NaiveDate,
    request: PutWorkEntryRequest,
) -> AppResult<WorkEntry> {
    // The request was validated, so the conversion only fails if that validation is bypassed.
    let hours = to_hundredths(request.hours).ok_or_else(|| field_error("hours", "range"))?;
    // A blank note is the same as no note.
    let note = request
        .note
        .as_deref()
        .map(str::trim)
        .filter(|note| !note.is_empty());

    if date > repository::user_today(db, user_id, Utc::now()).await? {
        return Err(field_error("date", "future"));
    }

    // An existing entry never needs the settings, so try it first; the insert then covers the
    // new-entry case and a concurrent create.
    if let Some(row) = repository::update(db, user_id, date, hours, note).await? {
        return Ok(row.into());
    }
    repository::insert_from_settings(db, user_id, date, hours, note)
        .await?
        .map(WorkEntry::from)
        .ok_or(AppError::WorkSettingsMissing)
}

/// Deletes the entry of `date`.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if the user has no entry on that day.
pub(super) async fn delete(db: &PgPool, user_id: Uuid, date: NaiveDate) -> AppResult<()> {
    if repository::delete(db, user_id, date).await? {
        Ok(())
    } else {
        Err(AppError::NotFound)
    }
}
