//! Request and response bodies of the work log endpoints, and the database row behind them.
//!
//! Hours cross the database boundary as whole hundredths of an hour, so pay is computed by
//! PostgreSQL with `numeric` arithmetic and never with floating point.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use crate::validation::to_hundredths;

/// Longest note, in characters.
const MAX_NOTE_LENGTH: u64 = 500;

/// How a day's hours compare with the base hours copied into the entry.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum Comparison {
    Above,
    Equal,
    Below,
}

/// One day's entry. The response body of `PUT /work-entries/{date}` and an item of the list.
#[derive(Debug, Serialize, PartialEq)]
pub(super) struct WorkEntry {
    pub date: NaiveDate,
    pub hours: f64,
    /// Minor units of the account currency, as copied when the entry was created.
    pub hourly_wage: i64,
    pub base_hours: f64,
    /// `hours × hourly_wage` rounded half up, in minor units.
    pub pay: i64,
    pub comparison: Comparison,
    pub note: Option<String>,
}

/// The stored entry; hours in hundredths of an hour and the pay already computed by the
/// database.
#[derive(Debug)]
pub(super) struct WorkEntryRow {
    pub date: NaiveDate,
    pub hours_hundredths: i32,
    pub hourly_wage: i64,
    pub base_hundredths: i32,
    pub pay: i64,
    pub note: Option<String>,
}

impl From<WorkEntryRow> for WorkEntry {
    fn from(row: WorkEntryRow) -> Self {
        let comparison = match row.hours_hundredths.cmp(&row.base_hundredths) {
            std::cmp::Ordering::Greater => Comparison::Above,
            std::cmp::Ordering::Equal => Comparison::Equal,
            std::cmp::Ordering::Less => Comparison::Below,
        };
        Self {
            date: row.date,
            hours: f64::from(row.hours_hundredths) / 100.0,
            hourly_wage: row.hourly_wage,
            base_hours: f64::from(row.base_hundredths) / 100.0,
            pay: row.pay,
            comparison,
            note: row.note,
        }
    }
}

/// The totals of one month.
#[derive(Debug, Serialize, PartialEq)]
pub(super) struct Summary {
    pub total_hours: f64,
    pub total_pay: i64,
    pub days_worked: usize,
}

impl Summary {
    /// Totals the rows. The pay is the sum of each day's rounded pay.
    pub(super) fn of(rows: &[WorkEntryRow]) -> Self {
        let hundredths: i32 = rows.iter().map(|row| row.hours_hundredths).sum();
        Self {
            // At most 31 days of 24 hours, so the sum fits an `i32`.
            total_hours: f64::from(hundredths) / 100.0,
            total_pay: rows.iter().map(|row| row.pay).sum(),
            days_worked: rows.len(),
        }
    }
}

/// Response body of `GET /work-entries`.
#[derive(Debug, Serialize)]
pub(super) struct MonthView {
    pub entries: Vec<WorkEntry>,
    pub summary: Summary,
}

/// Body of `PUT /work-entries/{date}`.
#[derive(Debug, Deserialize, Validate)]
pub(super) struct PutWorkEntryRequest {
    #[validate(custom(function = "valid_hours"))]
    pub hours: f64,
    #[validate(length(max = MAX_NOTE_LENGTH))]
    pub note: Option<String>,
}

/// Query of `GET /work-entries`.
#[derive(Debug, Deserialize)]
pub(super) struct ListQuery {
    /// The month as `YYYY-MM`.
    pub month: String,
}

/// `validator` rule: hours above 0 and at most 24 with at most two decimals. Code: `range`.
fn valid_hours(hours: f64) -> Result<(), ValidationError> {
    if to_hundredths(hours).is_none() {
        return Err(ValidationError::new("range"));
    }
    Ok(())
}

/// Parses `YYYY-MM` into the first day of that month, or `None` if it is not exactly that
/// shape or not a real month.
pub(super) fn parse_month(text: &str) -> Option<NaiveDate> {
    let bytes = text.as_bytes();
    let shaped = bytes.len() == 7
        && bytes[4] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || byte.is_ascii_digit());
    if !shaped {
        return None;
    }
    NaiveDate::parse_from_str(&format!("{text}-01"), "%Y-%m-%d").ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(hours: i32, base: i32, pay: i64) -> WorkEntryRow {
        WorkEntryRow {
            date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            hours_hundredths: hours,
            hourly_wage: 1,
            base_hundredths: base,
            pay,
            note: None,
        }
    }

    #[test]
    fn parse_month_accepts_only_year_dash_month() {
        assert_eq!(parse_month("2026-01"), NaiveDate::from_ymd_opt(2026, 1, 1));
        for bad in [
            "2026-13",
            "2026-00",
            "2026-1",
            "26-01",
            "2026-01-01",
            "",
            "abcd-ef",
            "+026-01",
        ] {
            assert_eq!(parse_month(bad), None, "{bad}");
        }
    }

    #[test]
    fn comparison_follows_hours_against_base() {
        assert_eq!(
            WorkEntry::from(row(801, 800, 0)).comparison,
            Comparison::Above
        );
        assert_eq!(
            WorkEntry::from(row(800, 800, 0)).comparison,
            Comparison::Equal
        );
        assert_eq!(
            WorkEntry::from(row(799, 800, 0)).comparison,
            Comparison::Below
        );
    }

    #[test]
    fn summary_totals_hours_pay_and_days() {
        let summary = Summary::of(&[row(125, 800, 37_501), row(800, 800, 240_008)]);

        assert_eq!(
            summary,
            Summary {
                total_hours: 9.25,
                total_pay: 277_509,
                days_worked: 2
            }
        );
    }
}
