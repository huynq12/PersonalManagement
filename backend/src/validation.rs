//! A JSON body extractor that reports failures in the API's error format.

use axum::{
    Json,
    extract::{FromRequest, Request},
};
use serde::de::DeserializeOwned;
use validator::Validate;

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
