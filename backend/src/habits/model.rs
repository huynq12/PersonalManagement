//! Request and response bodies of the habit endpoints, and the database rows behind them.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::validation::not_blank;

/// Longest habit name, in characters.
const MAX_NAME_LENGTH: u64 = 100;
/// ISO weekday numbers: 1 (Monday) to 7 (Sunday).
const WEEKDAY_RANGE: std::ops::RangeInclusive<i16> = 1..=7;

/// A habit with its current streak. Both the database row and the response body.
#[derive(Debug, Serialize)]
pub(crate) struct Habit {
    pub id: Uuid,
    pub name: String,
    /// ISO weekdays the habit applies to, sorted.
    pub weekdays: Vec<i16>,
    /// Whether the habit stays visible on low-energy days.
    pub low_energy: bool,
    /// Easier version shown on low-energy days. Always `null` when `low_energy` is `false`.
    pub low_energy_name: Option<String>,
    pub sort_order: i32,
    /// When the habit was archived, or `null` while it is active.
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    /// Consecutive scheduled days completed, counting back from today.
    pub current_streak: i32,
}

/// Body of `POST /habits`.
#[derive(Debug, Deserialize, Validate)]
pub(super) struct CreateHabitRequest {
    #[validate(length(max = MAX_NAME_LENGTH), custom(function = "not_blank"))]
    pub name: String,
    /// Defaults to every day.
    #[validate(custom(function = "valid_weekdays"))]
    pub weekdays: Option<Vec<i16>>,
    /// Defaults to `false`.
    pub low_energy: Option<bool>,
    /// Ignored unless `low_energy` is `true`; blank means "show the normal name".
    #[validate(length(max = MAX_NAME_LENGTH))]
    pub low_energy_name: Option<String>,
}

/// Body of `PATCH /habits/{id}`. Fields left out keep their value.
#[derive(Debug, Deserialize, Validate)]
pub(super) struct UpdateHabitRequest {
    #[validate(length(max = MAX_NAME_LENGTH), custom(function = "not_blank"))]
    pub name: Option<String>,
    #[validate(custom(function = "valid_weekdays"))]
    pub weekdays: Option<Vec<i16>>,
    pub low_energy: Option<bool>,
    /// Blank clears the easier name. Turning `low_energy` off clears it as well.
    #[validate(length(max = MAX_NAME_LENGTH))]
    pub low_energy_name: Option<String>,
    pub sort_order: Option<i32>,
    /// `true` archives the habit, `false` restores it.
    pub archived: Option<bool>,
}

/// Query of `GET /habits`.
#[derive(Debug, Deserialize)]
pub(super) struct ListHabitsQuery {
    #[serde(default)]
    pub include_archived: bool,
}

/// `validator` rule: at least one weekday, each between 1 and 7. Code: `weekdays`.
fn valid_weekdays(weekdays: &[i16]) -> Result<(), ValidationError> {
    let valid = !weekdays.is_empty()
        && weekdays
            .iter()
            .all(|weekday| WEEKDAY_RANGE.contains(weekday));
    if !valid {
        return Err(ValidationError::new("weekdays"));
    }
    Ok(())
}

/// Sorts the weekdays and drops duplicates, so `[5, 1, 1]` is stored as `[1, 5]`.
pub(super) fn normalize_weekdays(mut weekdays: Vec<i16>) -> Vec<i16> {
    weekdays.sort_unstable();
    weekdays.dedup();
    weekdays
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_weekdays_accepts_one_to_seven() {
        assert!(valid_weekdays(&[1, 7]).is_ok());
    }

    #[test]
    fn valid_weekdays_rejects_empty_and_out_of_range() {
        assert!(valid_weekdays(&[]).is_err());
        assert!(valid_weekdays(&[0]).is_err());
        assert!(valid_weekdays(&[8]).is_err());
    }

    #[test]
    fn normalize_weekdays_sorts_and_removes_duplicates() {
        assert_eq!(normalize_weekdays(vec![5, 1, 3, 1]), [1, 3, 5]);
    }
}
