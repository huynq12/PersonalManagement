//! Request and response bodies of the subject endpoints, and the database row behind them.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::validation::not_blank;

/// Longest subject name, in characters.
const MAX_NAME_LENGTH: u64 = 100;

/// A subject. Both the database row and the response body.
#[derive(Debug, Serialize)]
pub(crate) struct Subject {
    pub id: Uuid,
    pub name: String,
    /// Hex color in the form `#RRGGBB`.
    pub color: String,
    /// When the subject was archived, or `null` while it is active.
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Body of `POST /subjects`.
#[derive(Debug, Deserialize, Validate)]
pub(super) struct CreateSubjectRequest {
    #[validate(length(max = MAX_NAME_LENGTH), custom(function = "not_blank"))]
    pub name: String,
    #[validate(custom(function = "valid_color"))]
    pub color: String,
}

/// Body of `PATCH /subjects/{id}`. Fields left out keep their value.
#[derive(Debug, Deserialize, Validate)]
pub(super) struct UpdateSubjectRequest {
    #[validate(length(max = MAX_NAME_LENGTH), custom(function = "not_blank"))]
    pub name: Option<String>,
    #[validate(custom(function = "valid_color"))]
    pub color: Option<String>,
    /// `true` archives the subject, `false` restores it.
    pub archived: Option<bool>,
}

/// Query of `GET /subjects`.
#[derive(Debug, Deserialize)]
pub(super) struct ListSubjectsQuery {
    #[serde(default)]
    pub include_archived: bool,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_color_accepts_rrggbb_in_either_case() {
        assert!(valid_color("#3B82F6").is_ok());
        assert!(valid_color("#3b82f6").is_ok());
    }

    #[test]
    fn valid_color_rejects_other_shapes() {
        for color in [
            "",
            "#",
            "3B82F6",
            "#3B8",
            "#3B82F6F",
            "#GGGGGG",
            "#3B82F\u{e9}",
        ] {
            assert!(valid_color(color).is_err(), "{color:?} should be rejected");
        }
    }
}
