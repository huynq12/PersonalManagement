//! Response body of the day view.

use chrono::NaiveDate;
use serde::Serialize;
use uuid::Uuid;

use crate::tasks::model::Task;

/// A habit as shown on one day: whether it is done on that day.
#[derive(Debug, Serialize)]
pub(super) struct DayHabit {
    pub id: Uuid,
    pub name: String,
    pub completed: bool,
}

/// Body of `GET /days/{date}`: everything the Today screen shows for one date.
///
/// The menu and the study time described in SPEC.md are added together with those features.
#[derive(Debug, Serialize)]
pub(super) struct DayView {
    pub date: NaiveDate,
    /// Active habits scheduled for the date's weekday, in display order.
    pub habits: Vec<DayHabit>,
    /// The to-dos of the date, in display order.
    pub tasks: Vec<Task>,
}
