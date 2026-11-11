use axum::{Json, extract::State};
use serde::Serialize;

use crate::{
    AppState,
    error::{AppError, AppResult},
};

#[derive(Serialize)]
pub(super) struct Health {
    status: &'static str,
    database: &'static str,
}

/// `GET /api/v1/health` — reports whether the server is up and can reach the database.
pub(super) async fn health(State(state): State<AppState>) -> AppResult<Json<Health>> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await
        .map_err(|error| {
            tracing::warn!(%error, "health check could not reach the database");
            AppError::ServiceUnavailable
        })?;

    Ok(Json(Health {
        status: "ok",
        database: "ok",
    }))
}
