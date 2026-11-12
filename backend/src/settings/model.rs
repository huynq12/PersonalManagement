//! Request and response bodies of the settings endpoints.

use serde::{Deserialize, Serialize};
use validator::Validate;

/// Longest focus session, in minutes. Matches the longest session a client may start.
const MAX_FOCUS_MINUTES: i16 = 180;
const MAX_SHORT_BREAK_MINUTES: i16 = 60;
const MAX_LONG_BREAK_MINUTES: i16 = 120;
/// Most focus sessions before a long break.
const MAX_LONG_BREAK_EVERY: i16 = 12;

/// The user's Pomodoro settings. Both the database row and the body of `GET` and `PUT`.
#[derive(Debug, Serialize, Deserialize, Validate)]
pub(super) struct Settings {
    #[validate(range(min = 1, max = MAX_FOCUS_MINUTES))]
    pub focus_minutes: i16,
    #[validate(range(min = 1, max = MAX_SHORT_BREAK_MINUTES))]
    pub short_break_minutes: i16,
    #[validate(range(min = 1, max = MAX_LONG_BREAK_MINUTES))]
    pub long_break_minutes: i16,
    /// A long break follows every this many focus sessions.
    #[validate(range(min = 1, max = MAX_LONG_BREAK_EVERY))]
    pub long_break_every: i16,
}
