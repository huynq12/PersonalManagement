//! The query and response of the weekly statistics endpoint, and the database rows behind them.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::days::model::DayMode;

/// Query of `GET /stats/weekly`.
#[derive(Debug, Deserialize)]
pub(super) struct WeeklyStatsQuery {
    /// The Monday that starts the week; the current week when absent.
    pub week_start: Option<NaiveDate>,
}

/// One day of the week as read from the database.
#[derive(Debug)]
pub(super) struct DayRow {
    pub date: NaiveDate,
    pub is_low_energy: bool,
    pub study_seconds: i64,
    pub pomodoros: i64,
    pub habits_scheduled: i64,
    pub habits_done: i64,
    pub tasks_total: i64,
    pub tasks_done: i64,
}

impl DayRow {
    /// Habits and to-dos that count towards the checklist rate of the day.
    pub(super) fn checklist_total(&self) -> i64 {
        self.habits_scheduled + self.tasks_total
    }

    /// Habits and to-dos that are done.
    pub(super) fn checklist_done(&self) -> i64 {
        self.habits_done + self.tasks_done
    }
}

/// One day of the week in the response.
#[derive(Debug, Serialize)]
pub(super) struct DayStats {
    pub date: NaiveDate,
    pub mode: DayMode,
    /// Focus time of the completed sessions that started on the date.
    pub study_seconds: i64,
    /// Completed focus sessions.
    pub pomodoros: i64,
    pub habits_scheduled: i64,
    pub habits_done: i64,
    pub tasks_total: i64,
    pub tasks_done: i64,
    /// Done items divided by scheduled habits plus to-dos, from 0 to 1; `null` when the day has
    /// none of either.
    pub completion_rate: Option<f64>,
}

/// Study time of one subject over the week.
#[derive(Debug, Serialize)]
pub(super) struct SubjectStats {
    /// `null` for sessions without a subject, including those of a deleted subject.
    pub subject_id: Option<Uuid>,
    pub name: Option<String>,
    pub color: Option<String>,
    pub study_seconds: i64,
    pub pomodoros: i64,
}

/// The streaks of one active habit, as of today.
#[derive(Debug, Serialize)]
pub(super) struct HabitStreaks {
    pub id: Uuid,
    pub name: String,
    pub current_streak: i32,
    pub longest_streak: i32,
}

/// The figures of a whole week that are compared with the week before.
#[derive(Debug, Serialize)]
pub(super) struct WeekTotals {
    pub study_seconds: i64,
    pub pomodoros: i64,
    /// Items done over items scheduled across the week, from 0 to 1; `null` when there are none.
    pub completion_rate: Option<f64>,
}

/// Body of `GET /stats/weekly`.
#[derive(Debug, Serialize)]
pub(super) struct WeeklyStats {
    /// The Monday that starts the week.
    pub week_start: NaiveDate,
    /// The Sunday that ends the week.
    pub week_end: NaiveDate,
    /// The seven days from Monday to Sunday.
    pub days: Vec<DayStats>,
    /// Study time per subject, longest first.
    pub subjects: Vec<SubjectStats>,
    /// Streaks of the active habits in display order.
    pub habits: Vec<HabitStreaks>,
    pub totals: WeekTotals,
    /// The same figures for the seven days before `week_start`.
    pub previous_totals: WeekTotals,
}
