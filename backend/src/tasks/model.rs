//! Request and response bodies of the to-do endpoints, and the database row behind them.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::validation::not_blank;

/// Longest to-do title, in characters.
const MAX_TITLE_LENGTH: u64 = 200;

/// A to-do. Both the database row and the response body.
#[derive(Debug, Serialize)]
pub(crate) struct Task {
    pub id: Uuid,
    /// Calendar date in the user's time zone the to-do belongs to.
    pub date: NaiveDate,
    pub title: String,
    pub is_done: bool,
    /// When the to-do was marked done, or `null` while it is open.
    pub done_at: Option<DateTime<Utc>>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

/// Body of `POST /tasks`. Any date is allowed: to-dos can be planned ahead or added afterwards.
#[derive(Debug, Deserialize, Validate)]
pub(super) struct CreateTaskRequest {
    pub date: NaiveDate,
    #[validate(length(max = MAX_TITLE_LENGTH), custom(function = "not_blank"))]
    pub title: String,
}

/// Body of `PATCH /tasks/{id}`. Fields left out keep their value.
#[derive(Debug, Deserialize, Validate)]
pub(super) struct UpdateTaskRequest {
    #[validate(length(max = MAX_TITLE_LENGTH), custom(function = "not_blank"))]
    pub title: Option<String>,
    pub is_done: Option<bool>,
    pub date: Option<NaiveDate>,
    pub sort_order: Option<i32>,
}

/// Query of `GET /tasks`.
#[derive(Debug, Deserialize)]
pub(super) struct ListTasksQuery {
    pub date: NaiveDate,
}
