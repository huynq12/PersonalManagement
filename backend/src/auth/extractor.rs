//! Extractor that authenticates a request from its access token.

use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use uuid::Uuid;

use super::token;
use crate::{AppState, error::AppError};

const BEARER_PREFIX: &str = "Bearer ";

/// The authenticated user of a request.
///
/// Add it as a handler argument to require a valid `Authorization: Bearer <access token>`
/// header; requests without one are rejected with 401 before the handler runs.
#[derive(Debug, Clone, Copy)]
pub struct AuthUser {
    /// ID of the signed-in user. Every query for user-owned data must filter by it.
    pub id: Uuid,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let id = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix(BEARER_PREFIX))
            .and_then(|token| token::decode_access_token(&state.config.jwt_secret, token))
            .ok_or(AppError::Unauthorized)?;

        Ok(Self { id })
    }
}
