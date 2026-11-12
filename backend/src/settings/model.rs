//! Request and response body of the settings endpoints. The same shape is read and written.

use serde::{Deserialize, Serialize};
use validator::Validate;

/// The user's Pomodoro settings, all lengths in minutes.
///
/// Serves as the database row, the response body, and the `PUT` body, which replaces every
/// field. The ranges keep values sensible; `smallint` could hold far more.
#[derive(Debug, Serialize, Deserialize, Validate)]
pub(crate) struct PomodoroSettings {
    #[validate(range(min = 1, max = 180))]
    pub focus_minutes: i16,
    #[validate(range(min = 1, max = 60))]
    pub short_break_minutes: i16,
    #[validate(range(min = 1, max = 120))]
    pub long_break_minutes: i16,
    /// Focus sessions to finish before a long break.
    #[validate(range(min = 1, max = 12))]
    pub long_break_every: i16,
}
