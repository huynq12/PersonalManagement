//! Request and response bodies of the finance endpoints, and the database rows behind them.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::validation::{not_blank, present};

/// Longest category name, in characters.
const MAX_NAME_LENGTH: u64 = 100;
/// Longest transaction note, in characters.
const MAX_NOTE_LENGTH: usize = 500;
/// Largest amount, in minor units. Keeps every amount below 2^53, the largest integer
/// JavaScript represents exactly.
const MAX_AMOUNT: i64 = 1_000_000_000_000;

/// Whether a category, and every transaction in it, is money in or money out.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum Kind {
    Expense,
    Income,
}

impl Kind {
    /// The value stored in the `kind` column.
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Expense => "expense",
            Self::Income => "income",
        }
    }
}

/// A category. Both the database row and the response body.
#[derive(Debug, Serialize)]
pub(crate) struct Category {
    pub id: Uuid,
    /// `expense` or `income`.
    pub kind: String,
    pub name: String,
    /// Hex color in the form `#RRGGBB`.
    pub color: String,
    /// When the category was archived, or `null` while it is active.
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Body of `POST /finance-categories`.
#[derive(Debug, Deserialize, Validate)]
pub(super) struct CreateCategoryRequest {
    pub kind: Kind,
    #[validate(length(max = MAX_NAME_LENGTH), custom(function = "not_blank"))]
    pub name: String,
    #[validate(custom(function = "valid_color"))]
    pub color: String,
}

/// Body of `PATCH /finance-categories/{id}`. Fields left out keep their value; the kind cannot
/// change, so it is not accepted.
#[derive(Debug, Deserialize, Validate)]
pub(super) struct UpdateCategoryRequest {
    #[validate(length(max = MAX_NAME_LENGTH), custom(function = "not_blank"))]
    pub name: Option<String>,
    #[validate(custom(function = "valid_color"))]
    pub color: Option<String>,
    /// `true` archives the category, `false` restores it.
    pub archived: Option<bool>,
}

/// Query of `GET /finance-categories`.
#[derive(Debug, Deserialize)]
pub(super) struct ListCategoriesQuery {
    pub kind: Option<Kind>,
    #[serde(default)]
    pub include_archived: bool,
}

/// The category shown inside a transaction or a summary row.
#[derive(Debug, Serialize, PartialEq)]
pub(super) struct CategoryRef {
    pub id: Uuid,
    pub name: String,
    pub color: String,
}

/// A transaction with its category. The response body and an item of the month's list.
#[derive(Debug, Serialize)]
pub(super) struct Transaction {
    pub id: Uuid,
    /// The kind of the category: `expense` or `income`.
    pub kind: String,
    /// Minor units of the account currency.
    pub amount: i64,
    pub date: NaiveDate,
    pub note: Option<String>,
    pub category: CategoryRef,
    pub created_at: DateTime<Utc>,
}

/// A transaction joined with its category, as read from the database.
#[derive(Debug)]
pub(super) struct TransactionRow {
    pub id: Uuid,
    pub kind: String,
    pub amount: i64,
    pub date: NaiveDate,
    pub note: Option<String>,
    pub category_id: Uuid,
    pub category_name: String,
    pub category_color: String,
    pub created_at: DateTime<Utc>,
}

impl From<TransactionRow> for Transaction {
    fn from(row: TransactionRow) -> Self {
        Self {
            id: row.id,
            kind: row.kind,
            amount: row.amount,
            date: row.date,
            note: row.note,
            category: CategoryRef {
                id: row.category_id,
                name: row.category_name,
                color: row.category_color,
            },
            created_at: row.created_at,
        }
    }
}

/// Body of `POST /transactions`.
#[derive(Debug, Deserialize, Validate)]
pub(super) struct CreateTransactionRequest {
    pub category_id: Uuid,
    #[validate(custom(function = "valid_amount"))]
    pub amount: i64,
    pub date: NaiveDate,
    #[validate(custom(function = "valid_note"))]
    pub note: Option<String>,
}

/// Body of `PATCH /transactions/{id}`. Fields left out keep their value; `note: null` clears
/// the note, which is why it is an `Option<Option<_>>`.
#[derive(Debug, Deserialize, Validate)]
pub(super) struct UpdateTransactionRequest {
    pub category_id: Option<Uuid>,
    #[validate(custom(function = "valid_amount"))]
    pub amount: Option<i64>,
    pub date: Option<NaiveDate>,
    #[serde(default, deserialize_with = "present")]
    #[validate(custom(function = "valid_note"))]
    pub note: Option<Option<String>>,
}

/// Query of the endpoints that work on one month.
#[derive(Debug, Deserialize)]
pub(super) struct MonthQuery {
    /// The month as `YYYY-MM`.
    pub month: String,
}

/// One row of the per-category breakdown.
#[derive(Debug, Serialize, PartialEq)]
pub(super) struct CategoryTotal {
    pub kind: String,
    pub category: CategoryRef,
    pub total: i64,
}

/// The monthly expense budget and how much of it the month has used.
#[derive(Debug, Serialize, PartialEq)]
pub(super) struct Budget {
    pub amount: i64,
    pub spent: i64,
}

/// Response body of `GET /finances/summary`.
#[derive(Debug, Serialize)]
pub(super) struct Summary {
    pub income_total: i64,
    pub expense_total: i64,
    /// Income minus expenses; negative when the month spent more than it earned.
    pub balance: i64,
    pub by_category: Vec<CategoryTotal>,
    /// `null` when the user has not set a budget.
    pub budget: Option<Budget>,
}

/// `validator` rule: a color of the form `#RRGGBB`, with hex digits in either case.
/// Code: `color`.
fn valid_color(color: &str) -> Result<(), ValidationError> {
    let is_valid = color.len() == 7
        && color
            .strip_prefix('#')
            .is_some_and(|digits| digits.bytes().all(|byte| byte.is_ascii_hexdigit()));
    if !is_valid {
        return Err(ValidationError::new("color"));
    }
    Ok(())
}

/// `validator` rule: an amount from 1 to [`MAX_AMOUNT`]. Code: `range`.
fn valid_amount(amount: i64) -> Result<(), ValidationError> {
    if !(1..=MAX_AMOUNT).contains(&amount) {
        return Err(ValidationError::new("range"));
    }
    Ok(())
}

/// `validator` rule: a note of at most [`MAX_NOTE_LENGTH`] characters. Code: `length`.
fn valid_note(note: &str) -> Result<(), ValidationError> {
    if note.chars().count() > MAX_NOTE_LENGTH {
        return Err(ValidationError::new("length"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn valid_color_accepts_rrggbb_and_rejects_other_shapes() {
        assert!(valid_color("#3B82F6").is_ok());
        assert!(valid_color("#3b82f6").is_ok());
        for color in ["", "#", "3B82F6", "#3B8", "#GGGGGG"] {
            assert!(valid_color(color).is_err(), "{color:?}");
        }
    }

    #[test]
    fn valid_amount_is_positive_and_bounded() {
        assert!(valid_amount(1).is_ok());
        assert!(valid_amount(MAX_AMOUNT).is_ok());
        assert!(valid_amount(0).is_err());
        assert!(valid_amount(-1).is_err());
        assert!(valid_amount(MAX_AMOUNT + 1).is_err());
    }

    #[test]
    fn the_note_of_a_patch_can_be_absent_null_or_text() {
        let parse = |body: serde_json::Value| {
            serde_json::from_value::<UpdateTransactionRequest>(body).unwrap()
        };

        assert_eq!(parse(json!({})).note, None);
        assert_eq!(parse(json!({ "note": null })).note, Some(None));
        assert_eq!(
            parse(json!({ "note": "x" })).note,
            Some(Some("x".to_owned()))
        );
    }
}
