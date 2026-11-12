//! SQL for the auth feature.
//!
//! `users.email` is `citext`, which the SQLx macros cannot map directly, so queries cast it:
//! `email::text` when reading and `$n::text::citext` when comparing or writing. The second cast
//! matters: comparing `citext` with plain `text` would silently become case-sensitive.

use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use super::model::{StoredRefreshToken, User};

/// Values for a new `users` row.
pub(super) struct NewUser<'a> {
    pub email: &'a str,
    pub password_hash: Option<&'a str>,
    pub display_name: &'a str,
    pub locale: &'a str,
}

/// Inserts a user. Fails with a unique violation if the email is taken.
pub(super) async fn insert_user(
    executor: impl PgExecutor<'_>,
    new_user: NewUser<'_>,
) -> sqlx::Result<User> {
    sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (email, password_hash, display_name, locale)
        VALUES ($1::text::citext, $2, $3, $4)
        RETURNING id, email::text AS "email!", password_hash, display_name, timezone, locale
        "#,
        new_user.email,
        new_user.password_hash,
        new_user.display_name,
        new_user.locale,
    )
    .fetch_one(executor)
    .await
}

/// Profile fields to change; `None` keeps the stored value.
pub(super) struct ProfileChanges<'a> {
    pub display_name: Option<&'a str>,
    pub timezone: Option<&'a str>,
    pub locale: Option<&'a str>,
}

/// Updates the given profile fields. Returns `None` if the user does not exist.
pub(super) async fn update_profile(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    changes: ProfileChanges<'_>,
) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"
        UPDATE users
        SET display_name = COALESCE($2, display_name),
            timezone = COALESCE($3, timezone),
            locale = COALESCE($4, locale)
        WHERE id = $1
        RETURNING id, email::text AS "email!", password_hash, display_name, timezone, locale
        "#,
        user_id,
        changes.display_name,
        changes.timezone,
        changes.locale,
    )
    .fetch_optional(executor)
    .await
}

/// Creates the default Pomodoro settings row for a new user.
pub(super) async fn insert_user_settings(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
) -> sqlx::Result<()> {
    sqlx::query!("INSERT INTO user_settings (user_id) VALUES ($1)", user_id)
        .execute(executor)
        .await?;
    Ok(())
}

/// Finds a user by email, ignoring case.
pub(super) async fn find_user_by_email(
    executor: impl PgExecutor<'_>,
    email: &str,
) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"
        SELECT id, email::text AS "email!", password_hash, display_name, timezone, locale
        FROM users
        WHERE email = $1::text::citext
        "#,
        email,
    )
    .fetch_optional(executor)
    .await
}

/// Finds a user by ID.
pub(super) async fn find_user_by_id(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"
        SELECT id, email::text AS "email!", password_hash, display_name, timezone, locale
        FROM users
        WHERE id = $1
        "#,
        user_id,
    )
    .fetch_optional(executor)
    .await
}

/// Stores the hash of a newly issued refresh token.
pub(super) async fn insert_refresh_token(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    token_hash: &str,
    family_id: Uuid,
    expires_at: DateTime<Utc>,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, family_id, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
        user_id,
        token_hash,
        family_id,
        expires_at,
    )
    .execute(executor)
    .await?;
    Ok(())
}

/// Finds a refresh token by hash and locks its row until the transaction ends, so two
/// concurrent refreshes with the same token cannot both rotate it.
pub(super) async fn find_refresh_token_for_update(
    executor: impl PgExecutor<'_>,
    token_hash: &str,
) -> sqlx::Result<Option<StoredRefreshToken>> {
    sqlx::query_as!(
        StoredRefreshToken,
        r#"
        SELECT id, user_id, family_id, expires_at, revoked_at
        FROM refresh_tokens
        WHERE token_hash = $1
        FOR UPDATE
        "#,
        token_hash,
    )
    .fetch_optional(executor)
    .await
}

/// Revokes one refresh token.
pub(super) async fn revoke_refresh_token(
    executor: impl PgExecutor<'_>,
    id: Uuid,
) -> sqlx::Result<()> {
    sqlx::query!(
        "UPDATE refresh_tokens SET revoked_at = now() WHERE id = $1 AND revoked_at IS NULL",
        id,
    )
    .execute(executor)
    .await?;
    Ok(())
}

/// Revokes the refresh token with the given hash, if it exists and is still active.
pub(super) async fn revoke_refresh_token_by_hash(
    executor: impl PgExecutor<'_>,
    token_hash: &str,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
        UPDATE refresh_tokens SET revoked_at = now()
        WHERE token_hash = $1 AND revoked_at IS NULL
        "#,
        token_hash,
    )
    .execute(executor)
    .await?;
    Ok(())
}

/// Revokes every active refresh token descended from one sign-in.
pub(super) async fn revoke_refresh_token_family(
    executor: impl PgExecutor<'_>,
    family_id: Uuid,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
        UPDATE refresh_tokens SET revoked_at = now()
        WHERE family_id = $1 AND revoked_at IS NULL
        "#,
        family_id,
    )
    .execute(executor)
    .await?;
    Ok(())
}

/// Finds the user a provider account is linked to.
pub(super) async fn find_user_by_oauth_account(
    executor: impl PgExecutor<'_>,
    provider: &str,
    provider_user_id: &str,
) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"
        SELECT u.id, u.email::text AS "email!", u.password_hash, u.display_name, u.timezone,
               u.locale
        FROM users u
        JOIN oauth_accounts o ON o.user_id = u.id
        WHERE o.provider = $1 AND o.provider_user_id = $2
        "#,
        provider,
        provider_user_id,
    )
    .fetch_optional(executor)
    .await
}

/// Links a provider account to a user. Fails with a unique violation if the provider account
/// is already linked, or the user already linked another account of that provider.
pub(super) async fn insert_oauth_account(
    executor: impl PgExecutor<'_>,
    user_id: Uuid,
    provider: &str,
    provider_user_id: &str,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO oauth_accounts (user_id, provider, provider_user_id)
        VALUES ($1, $2, $3)
        "#,
        user_id,
        provider,
        provider_user_id,
    )
    .execute(executor)
    .await?;
    Ok(())
}
