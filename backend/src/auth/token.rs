//! Access tokens (short-lived JWTs) and refresh tokens (long-lived opaque random strings).

use anyhow::Context;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::config::Secret;

/// Lifetime of an access token: 15 minutes.
pub(super) const ACCESS_TOKEN_TTL_SECONDS: i64 = 15 * 60;
/// Lifetime of a refresh token: 30 days.
pub(super) const REFRESH_TOKEN_TTL_SECONDS: i64 = 30 * 24 * 60 * 60;

/// Marks a JWT as an access token, so other tokens signed with the same key are not accepted.
const ACCESS_AUDIENCE: &str = "access";
const REFRESH_TOKEN_BYTES: usize = 32;

#[derive(Debug, Serialize, Deserialize)]
struct AccessClaims {
    /// The user ID.
    sub: Uuid,
    aud: String,
    iat: i64,
    exp: i64,
}

/// Signs an access token for `user_id`.
///
/// # Errors
///
/// Returns an error if the token cannot be serialized or signed.
pub(super) fn encode_access_token(secret: &Secret, user_id: Uuid) -> anyhow::Result<String> {
    let now = Utc::now().timestamp();
    let claims = AccessClaims {
        sub: user_id,
        aud: ACCESS_AUDIENCE.to_owned(),
        iat: now,
        exp: now + ACCESS_TOKEN_TTL_SECONDS,
    };

    jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.expose().as_bytes()),
    )
    .context("failed to sign access token")
}

/// Returns the user ID of a valid access token, or `None` if the token is malformed, expired,
/// signed with another key, or not an access token.
pub(super) fn decode_access_token(secret: &Secret, token: &str) -> Option<Uuid> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&[ACCESS_AUDIENCE]);
    validation.set_required_spec_claims(&["exp", "sub", "aud"]);

    jsonwebtoken::decode::<AccessClaims>(
        token,
        &DecodingKey::from_secret(secret.expose().as_bytes()),
        &validation,
    )
    .ok()
    .map(|data| data.claims.sub)
}

/// A freshly generated refresh token.
pub(super) struct NewRefreshToken {
    /// The value sent to the browser. Never stored.
    pub token: String,
    /// The value stored in the database.
    pub hash: String,
}

/// Generates a refresh token from 256 bits of cryptographically secure randomness.
pub(super) fn generate_refresh_token() -> NewRefreshToken {
    let bytes: [u8; REFRESH_TOKEN_BYTES] = rand::random();
    let token = URL_SAFE_NO_PAD.encode(bytes);
    let hash = hash_refresh_token(&token);
    NewRefreshToken { token, hash }
}

/// Hashes a refresh token for storage and lookup.
///
/// The token is long and random, so a fast unsalted hash is enough: there is nothing to
/// brute-force, unlike a password.
pub(super) fn hash_refresh_token(token: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(token.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secret() -> Secret {
        Secret::new("unit-test-secret-unit-test-secret-unit")
    }

    fn sign(claims: &AccessClaims) -> String {
        jsonwebtoken::encode(
            &Header::new(Algorithm::HS256),
            claims,
            &EncodingKey::from_secret(secret().expose().as_bytes()),
        )
        .unwrap()
    }

    #[test]
    fn access_token_round_trips_the_user_id() {
        let user_id = Uuid::new_v4();

        let token = encode_access_token(&secret(), user_id).unwrap();

        assert_eq!(decode_access_token(&secret(), &token), Some(user_id));
    }

    #[test]
    fn access_token_signed_with_another_secret_is_rejected() {
        let token = encode_access_token(&Secret::new("x".repeat(32)), Uuid::new_v4()).unwrap();

        assert_eq!(decode_access_token(&secret(), &token), None);
    }

    #[test]
    fn expired_access_token_is_rejected() {
        let long_ago = Utc::now().timestamp() - 3600;
        let token = sign(&AccessClaims {
            sub: Uuid::new_v4(),
            aud: ACCESS_AUDIENCE.to_owned(),
            iat: long_ago - ACCESS_TOKEN_TTL_SECONDS,
            exp: long_ago,
        });

        assert_eq!(decode_access_token(&secret(), &token), None);
    }

    #[test]
    fn token_with_another_audience_is_rejected() {
        let now = Utc::now().timestamp();
        let token = sign(&AccessClaims {
            sub: Uuid::new_v4(),
            aud: "something-else".to_owned(),
            iat: now,
            exp: now + 60,
        });

        assert_eq!(decode_access_token(&secret(), &token), None);
    }

    #[test]
    fn refresh_tokens_are_unique_and_hash_deterministically() {
        let first = generate_refresh_token();
        let second = generate_refresh_token();

        assert_ne!(first.token, second.token);
        assert_ne!(first.token, first.hash);
        assert_eq!(first.hash, hash_refresh_token(&first.token));
    }
}
