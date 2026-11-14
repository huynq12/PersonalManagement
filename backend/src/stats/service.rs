//! Business rules for the weekly statistics: week validation, rates, and the week comparison.

use chrono::{Datelike, Days, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    model::{DayRow, DayStats, WeekTotals, WeeklyStats},
    repository,
};
use crate::{days::model::DayMode, error::AppResult, validation::field_error};

/// Days in a week; the comparison covers the same number of days before the week.
const WEEK_DAYS: u64 = 7;

/// Builds the statistics of the week that starts on `week_start` (the current week when `None`).
///
/// # Errors
///
/// Returns [`crate::error::AppError::Validation`] on the `week_start` field with code
/// `not_monday` when the date is not a Monday.
pub(super) async fn weekly(
    db: &PgPool,
    user_id: Uuid,
    week_start: Option<NaiveDate>,
) -> AppResult<WeeklyStats> {
    let today = repository::user_today(db, user_id, Utc::now()).await?;
    let week_start = match week_start {
        Some(date) if date.weekday().number_from_monday() != 1 => {
            return Err(field_error("week_start", "not_monday"));
        }
        Some(date) => date,
        None => monday_of(today),
    };
    let week_end = week_start + Days::new(WEEK_DAYS - 1);
    let previous_start = week_start - Days::new(WEEK_DAYS);

    let (rows, subjects, habits) = tokio::try_join!(
        // The previous week comes first in the same list, so one query serves both weeks.
        repository::list_days(db, user_id, previous_start, week_end),
        repository::list_subjects(db, user_id, week_start, week_end),
        repository::list_habit_streaks(db, user_id, today),
    )?;

    let (previous_rows, week_rows) = rows.split_at(rows.len() / 2);
    Ok(WeeklyStats {
        week_start,
        week_end,
        totals: totals(week_rows),
        previous_totals: totals(previous_rows),
        days: week_rows.iter().map(day_stats).collect(),
        subjects,
        habits,
    })
}

/// The Monday of the week that contains `date`.
fn monday_of(date: NaiveDate) -> NaiveDate {
    date - Days::new(u64::from(date.weekday().num_days_from_monday()))
}

/// Done items over all items, or `None` when there are no items.
#[allow(clippy::cast_precision_loss)] // Counts of a week are tiny compared with 2^53.
fn rate(done: i64, total: i64) -> Option<f64> {
    (total > 0).then(|| done as f64 / total as f64)
}

fn day_stats(row: &DayRow) -> DayStats {
    DayStats {
        date: row.date,
        mode: DayMode::from_low_energy(row.is_low_energy),
        study_seconds: row.study_seconds,
        pomodoros: row.pomodoros,
        habits_scheduled: row.habits_scheduled,
        habits_done: row.habits_done,
        tasks_total: row.tasks_total,
        tasks_done: row.tasks_done,
        completion_rate: rate(row.checklist_done(), row.checklist_total()),
    }
}

/// Sums a week. The rate pools every item of the week instead of averaging the daily rates, so
/// a busy day weighs more than an empty one.
fn totals(rows: &[DayRow]) -> WeekTotals {
    WeekTotals {
        study_seconds: rows.iter().map(|row| row.study_seconds).sum(),
        pomodoros: rows.iter().map(|row| row.pomodoros).sum(),
        completion_rate: rate(
            rows.iter().map(DayRow::checklist_done).sum(),
            rows.iter().map(DayRow::checklist_total).sum(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, day).unwrap()
    }

    #[test]
    fn monday_of_maps_every_weekday_to_the_monday_before_or_on_it() {
        // 2026-11-09 is a Monday.
        for day in 9..=15 {
            assert_eq!(monday_of(date(day)), date(9), "{}", date(day));
        }
        assert_eq!(monday_of(date(16)), date(16));
    }

    #[test]
    fn rate_is_none_without_items() {
        assert_eq!(rate(0, 0), None);
        assert_eq!(rate(1, 4), Some(0.25));
        assert_eq!(rate(0, 3), Some(0.0));
    }
}
