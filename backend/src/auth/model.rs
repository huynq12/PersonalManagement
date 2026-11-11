//! Request and response bodies of the auth endpoints, and the database rows behind them.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Interface languages a user can choose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum Locale {
    En,
    Vi,
}

impl Locale {
    /// The value stored in `users.locale`.
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Vi => "vi",
        }
    }
}

/// A row of the `users` table.
#[derive(Debug, Clone)]
pub(super) struct User {
    pub id: Uuid,
    pub email: String,
    /// `None` for accounts that only sign in with Google.
    pub password_hash: Option<String>,
    pub display_name: String,
    pub timezone: String,
    pub locale: String,
}

/// The columns of `refresh_tokens` needed to decide whether a token may be rotated.
#[derive(Debug)]
pub(super) struct StoredRefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub family_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

/// Body of `POST /auth/register`.
// No `Debug`: the struct holds a plain password and must never end up in a log line.
#[derive(Deserialize, Validate)]
pub(super) struct RegisterRequest {
    #[validate(email, length(max = 254))]
    pub email: String,
    /// Length only, no composition rules (NIST SP 800-63B). The upper bound caps hashing cost.
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    #[validate(length(min = 1, max = 100))]
    pub display_name: Option<String>,
    pub locale: Option<Locale>,
}

/// Body of `POST /auth/login`.
// No `Debug`, and no validation rules: a wrong value is simply an invalid credential.
#[derive(Deserialize, Validate)]
pub(super) struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// The public view of a user. Never includes the password hash.
#[derive(Debug, Serialize)]
pub(super) struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub timezone: String,
    pub locale: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            timezone: user.timezone,
            locale: user.locale,
        }
    }
}

/// Body returned by `POST /auth/login` and `POST /auth/refresh`.
#[derive(Debug, Serialize)]
pub(super) struct SessionResponse {
    pub access_token: String,
    pub token_type: &'static str,
    /// Lifetime of the access token in seconds.
    pub expires_in: i64,
    pub user: UserResponse,
}

/// Query of `GET /auth/google`.
#[derive(Debug, Deserialize)]
pub(super) struct GoogleStartQuery {
    /// Interface language to give the account if this sign-in creates one.
    pub locale: Option<Locale>,
}

/// Body returned by `POST /auth/google/link`.
#[derive(Debug, Serialize)]
pub(super) struct GoogleLinkResponse {
    /// The Google URL the browser must navigate to.
    pub authorization_url: String,
}
