//! Request extractors that report failures in the API's error format, and shared validators.
//!
//! Axum's own `Json`, `Path`, and `Query` extractors answer malformed input with a plain-text
//! body. The wrappers here turn those rejections into [`AppError`] so every error response has
//! the same JSON shape.

use axum::{
    Json,
    extract::{FromRequest, FromRequestParts, Path, Query, Request},
    http::request::Parts,
};
use serde::de::DeserializeOwned;
use validator::{Validate, ValidationError};

use crate::error::{AppError, FieldErrors};

/// Extracts a JSON body and runs its `validator` rules.
///
/// A body that cannot be parsed into `T` becomes [`AppError::InvalidRequest`]; a parsed body
/// that breaks a rule becomes [`AppError::Validation`] listing the failed rule codes per field.
#[derive(Debug)]
pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = AppError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(request, state)
            .await
            .map_err(|rejection| {
                tracing::debug!(%rejection, "rejected request body");
                AppError::InvalidRequest
            })?;

        value.validate().map_err(|errors| {
            let fields: FieldErrors = errors
                .field_errors()
                .into_iter()
                .map(|(field, failures)| {
                    let codes = failures
                        .iter()
                        .map(|failure| failure.code.to_string())
                        .collect();
                    (field.to_string(), codes)
                })
                .collect();
            AppError::Validation(fields)
        })?;

        Ok(Self(value))
    }
}

/// Extracts path parameters; a malformed one (such as an ID that is not a UUID or a date that
/// does not exist) becomes [`AppError::InvalidRequest`].
#[derive(Debug)]
pub struct ApiPath<T>(pub T);

impl<S, T> FromRequestParts<S> for ApiPath<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Send,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(value) =
            Path::<T>::from_request_parts(parts, state)
                .await
                .map_err(|rejection| {
                    tracing::debug!(%rejection, "rejected path parameters");
                    AppError::InvalidRequest
                })?;
        Ok(Self(value))
    }
}

/// Extracts the query string; a malformed one becomes [`AppError::InvalidRequest`].
#[derive(Debug)]
pub struct ApiQuery<T>(pub T);

impl<S, T> FromRequestParts<S> for ApiQuery<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(value) =
            Query::<T>::from_request_parts(parts, state)
                .await
                .map_err(|rejection| {
                    tracing::debug!(%rejection, "rejected query string");
                    AppError::InvalidRequest
                })?;
        Ok(Self(value))
    }
}

/// Builds an [`AppError::Validation`] for a single field and rule code. For rules that need
/// data a `validator` attribute cannot see, such as the current date.
#[must_use]
pub fn field_error(field: &str, code: &str) -> AppError {
    AppError::Validation(FieldErrors::from([(
        field.to_owned(),
        vec![code.to_owned()],
    )]))
}

/// `validator` rule: the text must contain something other than whitespace. Code: `blank`.
///
/// # Errors
///
/// Returns a [`ValidationError`] if the text is empty or only whitespace.
pub fn not_blank(text: &str) -> Result<(), ValidationError> {
    if text.trim().is_empty() {
        return Err(ValidationError::new("blank"));
    }
    Ok(())
}

/// `validator` rule: the text must be an IANA time zone name such as `Asia/Ho_Chi_Minh`.
/// Code: `timezone`.
///
/// # Errors
///
/// Returns a [`ValidationError`] if the name is not in the IANA database.
pub fn valid_timezone(name: &str) -> Result<(), ValidationError> {
    if name.parse::<chrono_tz::Tz>().is_err() {
        return Err(ValidationError::new("timezone"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_timezone_accepts_iana_names_only() {
        assert!(valid_timezone("Asia/Ho_Chi_Minh").is_ok());
        assert!(valid_timezone("UTC").is_ok());
        for name in ["", "Mars/Olympus", "UTC+7", " Asia/Ho_Chi_Minh"] {
            assert!(valid_timezone(name).is_err(), "{name:?} should be rejected");
        }
    }

    #[test]
    fn not_blank_rejects_empty_and_whitespace_only_text() {
        assert!(not_blank("").is_err());
        assert!(not_blank(" \t\n").is_err());
        assert!(not_blank(" a ").is_ok());
    }
}
