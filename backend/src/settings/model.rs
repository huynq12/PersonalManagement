//! Request and response bodies of the settings endpoints.
//!
//! Money fields are integers in the account currency's minor unit. `base_work_hours` is stored
//! as `numeric(4,2)`; it crosses the database boundary as a whole number of hundredths of an
//! hour, so no floating point is ever stored.

use serde::{Deserialize, Deserializer, Serialize};
use validator::{Validate, ValidationError};

use crate::validation::to_hundredths;

/// Largest hourly wage or monthly budget, in minor units. Keeps every amount below 2^53, the
/// largest integer JavaScript represents exactly.
const MAX_MONEY: i64 = 1_000_000_000_000;
/// Longest Pomodoro focus or break, in minutes.
const MAX_MINUTES: i16 = 240;
/// Largest number of focus sessions between long breaks.
const MAX_LONG_BREAK_EVERY: i16 = 12;

/// The settings of one user. The response body of both endpoints.
#[derive(Debug, Serialize, PartialEq)]
pub(super) struct Settings {
    pub focus_minutes: i16,
    pub short_break_minutes: i16,
    pub long_break_minutes: i16,
    pub long_break_every: i16,
    pub hourly_wage: Option<i64>,
    /// Hours with up to two decimals.
    pub base_work_hours: Option<f64>,
    pub monthly_budget: Option<i64>,
}

/// The settings row as stored; `base_work_hours` in hundredths of an hour.
#[derive(Debug)]
pub(super) struct SettingsRow {
    pub focus_minutes: i16,
    pub short_break_minutes: i16,
    pub long_break_minutes: i16,
    pub long_break_every: i16,
    pub hourly_wage: Option<i64>,
    pub base_work_hundredths: Option<i32>,
    pub monthly_budget: Option<i64>,
}

impl From<SettingsRow> for Settings {
    fn from(row: SettingsRow) -> Self {
        Self {
            focus_minutes: row.focus_minutes,
            short_break_minutes: row.short_break_minutes,
            long_break_minutes: row.long_break_minutes,
            long_break_every: row.long_break_every,
            hourly_wage: row.hourly_wage,
            base_work_hours: row.base_work_hundredths.map(|h| f64::from(h) / 100.0),
            monthly_budget: row.monthly_budget,
        }
    }
}

/// Body of `PATCH /settings`.
///
/// Fields left out keep their value. The nullable settings are `Option<Option<_>>`: the outer
/// layer says whether the field was sent, the inner one holds its value, and `null` clears it.
/// The Pomodoro fields cannot be cleared, so a `null` for them is a malformed request.
#[derive(Debug, Deserialize, Validate)]
pub(super) struct UpdateSettingsRequest {
    #[serde(default, deserialize_with = "not_null")]
    #[validate(range(min = 1, max = MAX_MINUTES))]
    pub focus_minutes: Option<i16>,
    #[serde(default, deserialize_with = "not_null")]
    #[validate(range(min = 1, max = MAX_MINUTES))]
    pub short_break_minutes: Option<i16>,
    #[serde(default, deserialize_with = "not_null")]
    #[validate(range(min = 1, max = MAX_MINUTES))]
    pub long_break_minutes: Option<i16>,
    #[serde(default, deserialize_with = "not_null")]
    #[validate(range(min = 1, max = MAX_LONG_BREAK_EVERY))]
    pub long_break_every: Option<i16>,
    #[serde(default, deserialize_with = "present")]
    #[validate(custom(function = "valid_money"))]
    pub hourly_wage: Option<Option<i64>>,
    #[serde(default, deserialize_with = "present")]
    #[validate(custom(function = "valid_hours"))]
    pub base_work_hours: Option<Option<f64>>,
    #[serde(default, deserialize_with = "present")]
    #[validate(custom(function = "valid_money"))]
    pub monthly_budget: Option<Option<i64>>,
}

/// Deserializes a field that may be absent, `null`, or a value into `None`, `Some(None)`, or
/// `Some(Some(value))`. Use together with `#[serde(default)]`.
fn present<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

/// Deserializes a field that may be absent or a value, and rejects `null`. Use together with
/// `#[serde(default)]`.
fn not_null<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    T::deserialize(deserializer).map(Some)
}

/// `validator` rule: an amount from 1 to [`MAX_MONEY`]. Code: `range`.
// `validator` skips absent and `null` values, so the function only sees an actual amount.
fn valid_money(amount: i64) -> Result<(), ValidationError> {
    if !(1..=MAX_MONEY).contains(&amount) {
        return Err(ValidationError::new("range"));
    }
    Ok(())
}

/// `validator` rule: hours above 0 and at most 24 with at most two
/// decimals. Code: `range`.
fn valid_hours(hours: f64) -> Result<(), ValidationError> {
    if to_hundredths(hours).is_none() {
        return Err(ValidationError::new("range"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn parse(body: serde_json::Value) -> Result<UpdateSettingsRequest, serde_json::Error> {
        serde_json::from_value(body)
    }

    #[test]
    fn absent_null_and_value_are_told_apart() {
        let absent = parse(json!({})).unwrap();
        let cleared = parse(json!({ "hourly_wage": null })).unwrap();
        let set = parse(json!({ "hourly_wage": 5 })).unwrap();

        assert_eq!(absent.hourly_wage, None);
        assert_eq!(cleared.hourly_wage, Some(None));
        assert_eq!(set.hourly_wage, Some(Some(5)));
    }

    #[test]
    fn null_is_rejected_for_a_field_that_cannot_be_cleared() {
        assert!(parse(json!({ "focus_minutes": null })).is_err());
        assert_eq!(
            parse(json!({ "focus_minutes": 30 })).unwrap().focus_minutes,
            Some(30)
        );
        assert_eq!(parse(json!({})).unwrap().focus_minutes, None);
    }

    #[test]
    fn valid_money_checks_the_range() {
        assert!(valid_money(1).is_ok());
        assert!(valid_money(MAX_MONEY).is_ok());
        assert!(valid_money(0).is_err());
        assert!(valid_money(MAX_MONEY + 1).is_err());
    }
}
