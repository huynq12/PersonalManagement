//! The API error type and its JSON representation.

use std::collections::BTreeMap;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};

/// Validation failures per field: field name to the list of failed rule codes.
pub type FieldErrors = BTreeMap<String, Vec<String>>;

/// Every failure a handler can return.
///
/// Serialized as `{ "error": { "code": "...", "message": "...", "details": ... } }`. `code` is
/// a stable identifier the frontend translates for users; `message` is English and
/// developer-facing.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// The route or resource does not exist, or belongs to another user.
    #[error("resource not found")]
    NotFound,
    /// The request body is missing, is not valid JSON, or has the wrong shape.
    #[error("request body is malformed")]
    InvalidRequest,
    /// The request was well formed but one or more fields broke a validation rule.
    #[error("one or more fields are invalid")]
    Validation(FieldErrors),
    /// The access token is missing, malformed, expired, or belongs to a deleted user.
    #[error("authentication required")]
    Unauthorized,
    /// The email and password do not match an account.
    #[error("invalid email or password")]
    InvalidCredentials,
    /// The refresh token is missing, unknown, expired, or revoked.
    #[error("refresh token is invalid")]
    InvalidRefreshToken,
    /// Another account already uses this email.
    #[error("email is already registered")]
    EmailTaken,
    /// Google sign-in could not be completed: bad state, rejected code, or invalid ID token.
    #[error("google sign-in failed")]
    GoogleSignInFailed,
    /// The Google account's email is not verified by Google.
    #[error("google account email is not verified")]
    GoogleEmailUnverified,
    /// The Google email matches a password account; the user must sign in and link Google.
    #[error("email is already registered with a password")]
    EmailAlreadyRegistered,
    /// The Google account is linked to a different user, or the user already linked another one.
    #[error("google account is already linked")]
    GoogleAccountAlreadyLinked,
    /// Another subject of the same user already has this name, ignoring case.
    #[error("subject name is already in use")]
    SubjectNameTaken,
    /// The user already has a running study session.
    #[error("a study session is already running")]
    StudySessionAlreadyRunning,
    /// The study session already completed or was cancelled.
    #[error("study session is not running")]
    StudySessionNotRunning,
    /// The day mode cannot change because a habit is already checked for today.
    #[error("day mode cannot change while a habit is checked")]
    DayModeLocked,
    /// A new work entry needs the hourly wage and base hours, and one of them is not set.
    #[error("hourly wage or base work hours are not set")]
    WorkSettingsMissing,
    /// A dependency such as the database is unreachable.
    #[error("service temporarily unavailable")]
    ServiceUnavailable,
    /// An unexpected database failure.
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    /// Any other unexpected failure.
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    /// The stable snake_case identifier the frontend translates.
    #[must_use]
    pub fn code(&self) -> &'static str {
        self.status_and_code().1
    }

    fn status_and_code(&self) -> (StatusCode, &'static str) {
        match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            Self::InvalidRequest => (StatusCode::BAD_REQUEST, "invalid_request"),
            Self::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, "validation_failed"),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized"),
            Self::InvalidCredentials => (StatusCode::UNAUTHORIZED, "invalid_credentials"),
            Self::InvalidRefreshToken => (StatusCode::UNAUTHORIZED, "invalid_refresh_token"),
            Self::EmailTaken => (StatusCode::CONFLICT, "email_taken"),
            Self::GoogleSignInFailed => (StatusCode::BAD_REQUEST, "google_sign_in_failed"),
            Self::GoogleEmailUnverified => (StatusCode::FORBIDDEN, "google_email_unverified"),
            Self::EmailAlreadyRegistered => (StatusCode::CONFLICT, "email_already_registered"),
            Self::GoogleAccountAlreadyLinked => {
                (StatusCode::CONFLICT, "google_account_already_linked")
            }
            Self::SubjectNameTaken => (StatusCode::CONFLICT, "subject_name_taken"),
            Self::StudySessionAlreadyRunning => {
                (StatusCode::CONFLICT, "study_session_already_running")
            }
            Self::StudySessionNotRunning => (StatusCode::CONFLICT, "study_session_not_running"),
            Self::DayModeLocked => (StatusCode::CONFLICT, "day_mode_locked"),
            Self::WorkSettingsMissing => (StatusCode::CONFLICT, "work_settings_missing"),
            Self::ServiceUnavailable => (StatusCode::SERVICE_UNAVAILABLE, "service_unavailable"),
            Self::Database(_) | Self::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error")
            }
        }
    }

    /// Logs unexpected failures. Call exactly once per error, where it leaves the application.
    pub fn log_if_unexpected(&self) {
        if matches!(self, Self::Database(_) | Self::Internal(_)) {
            tracing::error!(error = ?self, "unhandled error");
        }
    }

    fn details(&self) -> Option<Value> {
        match self {
            Self::Validation(fields) => Some(json!({ "fields": fields })),
            _ => None,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = self.status_and_code();
        self.log_if_unexpected();

        // Internal details stay in the log and are never sent to the client.
        let message = if status == StatusCode::INTERNAL_SERVER_ERROR {
            "internal server error".to_owned()
        } else {
            self.to_string()
        };

        let mut error = json!({ "code": code, "message": message });
        if let Some(details) = self.details() {
            error["details"] = details;
        }

        (status, Json(json!({ "error": error }))).into_response()
    }
}

/// Whether a database error is a unique constraint violation, so a service can turn it into
/// the matching "already taken" [`AppError`].
pub(crate) fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .is_some_and(sqlx::error::DatabaseError::is_unique_violation)
}

/// Result alias used by handlers.
pub type AppResult<T> = Result<T, AppError>;
