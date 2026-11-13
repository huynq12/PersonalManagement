//! Request and response bodies of the study session endpoints, and the database row behind them.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// A focus session. Both the database row and the response body.
#[derive(Debug, Serialize)]
pub(crate) struct StudySession {
    pub id: Uuid,
    /// The subject studied, or `null` if none was chosen or it was deleted.
    pub subject_id: Option<Uuid>,
    /// `running`, `completed`, or `cancelled`.
    pub status: String,
    pub planned_minutes: i16,
    pub started_at: DateTime<Utc>,
    /// When the session completed or was cancelled; `null` while it runs.
    pub ended_at: Option<DateTime<Utc>>,
    /// Elapsed focus time, capped at the planned length; set only on completed sessions.
    pub duration_seconds: Option<i32>,
    /// Start date in the user's time zone.
    pub local_date: NaiveDate,
}

/// Body of `POST /study-sessions`.
#[derive(Debug, Deserialize, Validate)]
pub(super) struct StartSessionRequest {
    pub subject_id: Option<Uuid>,
}

/// Query of `GET /study-sessions`: an inclusive range of local dates.
#[derive(Debug, Deserialize)]
pub(super) struct ListSessionsQuery {
    pub from: NaiveDate,
    pub to: NaiveDate,
}
