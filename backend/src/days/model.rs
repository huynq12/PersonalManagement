//! Request and response bodies of the day endpoints.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::tasks::model::Task;

/// Whether a day is a normal day or a low-energy day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DayMode {
    Normal,
    LowEnergy,
}

impl DayMode {
    /// The mode of a day, given whether it is marked as a low-energy day.
    pub(crate) fn from_low_energy(is_low_energy: bool) -> Self {
        if is_low_energy {
            Self::LowEnergy
        } else {
            Self::Normal
        }
    }
}

/// Body of `PUT /days/{date}/mode`.
#[derive(Debug, Deserialize, Validate)]
pub(super) struct SetDayModeRequest {
    pub mode: DayMode,
}

/// A habit as shown on one day: whether it is done on that day.
#[derive(Debug, Serialize)]
pub(super) struct DayHabit {
    pub id: Uuid,
    /// The easier name on a low-energy day when the habit has one, otherwise the habit's name.
    pub name: String,
    pub completed: bool,
}

/// The mode of a day and whether the user may change it.
#[derive(Debug)]
pub(super) struct DayModeState {
    pub is_low_energy: bool,
    /// The date is not today, or a habit is already checked for today.
    pub locked: bool,
}

/// The hours worked on a day and the pay they earn.
#[derive(Debug, Serialize)]
pub(super) struct DayWork {
    pub hours: f64,
    /// `hours × hourly_wage` rounded half up, in minor units of the account currency.
    pub pay: i64,
}

/// Body of `GET /days/{date}`: everything the Today screen shows for one date.
///
/// The menu described in SPEC.md is added together with the meal plan.
#[derive(Debug, Serialize)]
pub(super) struct DayView {
    pub date: NaiveDate,
    pub mode: DayMode,
    /// Whether `PUT /days/{date}/mode` would currently be refused.
    pub mode_locked: bool,
    /// Active habits scheduled for the date, in display order. On a low-energy day only the
    /// habits kept for such days.
    pub habits: Vec<DayHabit>,
    /// The to-dos of the date, in display order.
    pub tasks: Vec<Task>,
    /// The work entry of the date, or `None` when no hours were logged.
    pub work: Option<DayWork>,
    /// Total of the date's expense transactions in minor units; 0 when nothing was spent.
    pub expense_total: i64,
    /// Total time of the date's completed study sessions, in seconds; 0 when there are none.
    pub study_seconds: i64,
}
