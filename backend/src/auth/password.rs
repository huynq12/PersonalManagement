//! Password hashing with Argon2id.

use std::sync::LazyLock;

use anyhow::{Context, anyhow};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};

/// A valid hash of a throwaway password, verified against when the account does not exist so
/// that a login attempt takes the same time whether or not the email is registered.
static DUMMY_HASH: LazyLock<Option<String>> =
    LazyLock::new(|| hash_blocking("dummy password for timing").ok());

/// Hashes a password into a PHC string (`$argon2id$v=19$...`) with a random salt.
///
/// # Errors
///
/// Returns an error if hashing fails or the blocking task is cancelled.
pub(super) async fn hash(password: String) -> anyhow::Result<String> {
    // Argon2 is deliberately slow, so it must not run on the async worker threads.
    tokio::task::spawn_blocking(move || hash_blocking(&password))
        .await
        .context("password hashing task failed")?
}

/// Checks a password against a stored PHC string.
///
/// # Errors
///
/// Returns an error if the stored hash is malformed or the blocking task is cancelled.
pub(super) async fn verify(password: String, stored_hash: String) -> anyhow::Result<bool> {
    tokio::task::spawn_blocking(move || verify_blocking(&password, &stored_hash))
        .await
        .context("password verification task failed")?
}

/// Spends the time of one verification without checking a real account.
pub(super) async fn verify_dummy(password: String) {
    if let Some(dummy) = DUMMY_HASH.as_ref() {
        // The outcome is irrelevant: only the time spent matters.
        let _ = verify(password, dummy.clone()).await;
    }
}

fn hash_blocking(password: &str) -> anyhow::Result<String> {
    let hash = Argon2::default()
        .hash_password(password.as_bytes())
        .map_err(|error| anyhow!("failed to hash password: {error}"))?;
    Ok(hash.to_string())
}

fn verify_blocking(password: &str, stored_hash: &str) -> anyhow::Result<bool> {
    let parsed = PasswordHash::new(stored_hash)
        .map_err(|error| anyhow!("stored password hash is malformed: {error}"))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_argon2id_and_verifies_only_the_original_password() {
        let hash = hash_blocking("correct horse").unwrap();

        assert!(hash.starts_with("$argon2id$"));
        assert!(verify_blocking("correct horse", &hash).unwrap());
        assert!(!verify_blocking("wrong horse", &hash).unwrap());
    }

    #[test]
    fn hashing_the_same_password_twice_uses_different_salts() {
        assert_ne!(
            hash_blocking("correct horse").unwrap(),
            hash_blocking("correct horse").unwrap()
        );
    }

    #[test]
    fn verify_reports_malformed_stored_hash_as_error() {
        assert!(verify_blocking("anything", "not a phc string").is_err());
    }
}
