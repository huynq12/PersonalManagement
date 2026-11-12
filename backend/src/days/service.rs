//! Assembles the day view from the habits and to-dos features.

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use super::{model::DayView, repository};
use crate::{error::AppResult, tasks};

/// Loads the scheduled habits and the to-dos of one date.
pub(super) async fn day_view(db: &PgPool, user_id: Uuid, date: NaiveDate) -> AppResult<DayView> {
    // The two queries are independent, so run them concurrently.
    let (habits, tasks) = tokio::try_join!(
        repository::list_day_habits(db, user_id, date),
        tasks::repository::list_tasks(db, user_id, date),
    )?;

    Ok(DayView {
        date,
        habits,
        tasks,
    })
}
