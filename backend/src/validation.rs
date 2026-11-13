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

const MAX_HOURS_PER_DAY: f64 = 24.0;

/// Converts hours to whole hundredths of an hour, or `None` if the value is out of range or
/// has more than two decimals.
pub(crate) fn to_hundredths(hours: f64) -> Option<i32> {
    if !hours.is_finite() || hours <= 0.0 || hours > MAX_HOURS_PER_DAY {
        return None;
    }
    let scaled = hours * 100.0;
    let rounded = scaled.round();
    // Tolerates binary representation error such as 7.25 * 100 = 724.9999999999999.
    if (scaled - rounded).abs() > 1e-6 {
        return None;
    }
    // The range check above keeps `rounded` within 1..=2400.
    #[allow(clippy::cast_possible_truncation)]
    Some(rounded as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_blank_rejects_empty_and_whitespace_only_text() {
        assert!(not_blank("").is_err());
        assert!(not_blank(" \t\n").is_err());
        assert!(not_blank(" a ").is_ok());
    }

    #[test]
    fn to_hundredths_accepts_up_to_two_decimals_in_range() {
        assert_eq!(to_hundredths(8.0), Some(800));
        assert_eq!(to_hundredths(7.25), Some(725));
        assert_eq!(to_hundredths(0.01), Some(1));
        assert_eq!(to_hundredths(24.0), Some(2400));
        assert_eq!(to_hundredths(1.1), Some(110));
    }

    #[test]
    fn to_hundredths_rejects_everything_else() {
        for hours in [0.0, -1.0, 24.01, 7.255, f64::NAN, f64::INFINITY] {
            assert_eq!(to_hundredths(hours), None, "{hours}");
        }
    }
}
