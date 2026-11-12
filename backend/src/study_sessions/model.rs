//! Request and response bodies of the study session endpoints, and the database row behind them.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Longest focus session a client may ask for, in minutes.
const MAX_PLANNED_MINUTES: i16 = 180;

/// A focus session. Both the database row and the response body.
#[derive(Debug, Serialize)]
pub(super) struct StudySession {
    pub id: Uuid,
    /// The subject studied; `null` for a session without one, or once the subject is deleted.
    pub subject_id: Option<Uuid>,
    /// `running`, `completed`, or `cancelled`.
    pub status: String,
    pub planned_minutes: i16,
    pub started_at: DateTime<Utc>,
    /// When the session was completed or cancelled; `null` while it is running.
    pub ended_at: Option<DateTime<Utc>>,
    /// Time studied, computed by the server; only set on completed sessions.
    pub duration_seconds: Option<i32>,
    /// Start date in the user's time zone.
    pub local_date: NaiveDate,
}

/// Body of `POST /study-sessions`.
#[derive(Debug, Deserialize, Validate)]
pub(super) struct StartSessionRequest {
    pub subject_id: Option<Uuid>,
    /// Length of the focus period; the user's `focus_minutes` setting when left out.
    #[validate(range(min = 1, max = MAX_PLANNED_MINUTES))]
    pub planned_minutes: Option<i16>,
}

/// Query of `GET /study-sessions`: an inclusive range of local dates.
#[derive(Debug, Deserialize)]
pub(super) struct ListSessionsQuery {
    pub from: NaiveDate,
    pub to: NaiveDate,
}
