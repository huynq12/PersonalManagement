//! Business rules of the auth feature.

use chrono::{Duration, Utc};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use super::{
    google::{self, CallbackQuery, Flow, GoogleClient, GoogleProfile},
    model::{Locale, LoginRequest, RegisterRequest, UpdateMeRequest, User},
    password,
    repository::{self, NewUser, UserChanges},
    token,
};
use crate::{
    config::Config,
    error::{AppError, AppResult, is_unique_violation},
    validation::field_error,
};

/// A signed-in session: what the handler turns into a response body and a cookie.
pub(super) struct Session {
    pub user: User,
    pub access_token: String,
    /// The raw refresh token, to be sent to the browser as a cookie.
    pub refresh_token: String,
}

/// Registers an account with an email and a password. Does not sign the user in.
///
/// # Errors
///
/// Returns [`AppError::EmailTaken`] if another account uses the email.
pub(super) async fn register(db: &PgPool, request: RegisterRequest) -> AppResult<User> {
    let email = request.email.trim();
    let display_name = request
        .display_name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| default_display_name(email));
    let locale = request.locale.unwrap_or(Locale::En);
    let password_hash = password::hash(request.password).await?;

    let mut tx = db.begin().await?;
    let user = create_user(
        &mut tx,
        NewUser {
            email,
            password_hash: Some(&password_hash),
            display_name,
            locale: locale.as_str(),
        },
    )
    .await?;
    tx.commit().await?;

    tracing::info!(user_id = %user.id, "user registered");
    Ok(user)
}

/// Inserts a user together with their default settings. Run inside a transaction.
///
/// # Errors
///
/// Returns [`AppError::EmailTaken`] if another account uses the email.
pub(super) async fn create_user(conn: &mut PgConnection, new_user: NewUser<'_>) -> AppResult<User> {
    let user = repository::insert_user(&mut *conn, new_user)
        .await
        .map_err(|error| {
            if is_unique_violation(&error) {
                AppError::EmailTaken
            } else {
                AppError::Database(error)
            }
        })?;
    repository::insert_user_settings(&mut *conn, user.id).await?;
    Ok(user)
}

/// Signs a user in with their email and password.
///
/// # Errors
///
/// Returns [`AppError::InvalidCredentials`] if the email is unknown, the account has no
/// password (Google-only), or the password is wrong. The three cases are indistinguishable
/// to the caller, in both the response and the time it takes.
pub(super) async fn login(
    db: &PgPool,
    config: &Config,
    request: LoginRequest,
) -> AppResult<Session> {
    let user = repository::find_user_by_email(db, request.email.trim()).await?;

    let Some((user, password_hash)) =
        user.and_then(|user| user.password_hash.clone().map(|hash| (user, hash)))
    else {
        password::verify_dummy(request.password).await;
        return Err(AppError::InvalidCredentials);
    };

    if !password::verify(request.password, password_hash).await? {
        return Err(AppError::InvalidCredentials);
    }

    let mut conn = db.acquire().await?;
    let session = start_session(&mut conn, config, user).await?;
    tracing::info!(user_id = %session.user.id, "user signed in");
    Ok(session)
}

/// Starts a new sign-in for `user`: a new refresh token family and an access token.
///
/// # Errors
///
/// Returns an error if the token cannot be stored or signed.
pub(super) async fn start_session(
    conn: &mut PgConnection,
    config: &Config,
    user: User,
) -> AppResult<Session> {
    issue_tokens(conn, config, user, Uuid::new_v4()).await
}

/// Exchanges a refresh token for a new access token and a new refresh token (rotation).
///
/// Presenting a token that was already rotated or revoked is treated as theft: every token
/// of its family is revoked, which signs out both the thief and the legitimate browser.
///
/// # Errors
///
/// Returns [`AppError::InvalidRefreshToken`] if the token is unknown, expired, or revoked.
pub(super) async fn refresh(
    db: &PgPool,
    config: &Config,
    refresh_token: &str,
) -> AppResult<Session> {
    let token_hash = token::hash_refresh_token(refresh_token);

    let mut tx = db.begin().await?;
    let Some(stored) = repository::find_refresh_token_for_update(&mut *tx, &token_hash).await?
    else {
        return Err(AppError::InvalidRefreshToken);
    };

    if stored.revoked_at.is_some() {
        repository::revoke_refresh_token_family(&mut *tx, stored.family_id).await?;
        // Commit before returning: dropping the transaction would undo the revocation.
        tx.commit().await?;
        tracing::warn!(
            user_id = %stored.user_id,
            family_id = %stored.family_id,
            "revoked refresh token was reused; revoked its whole family"
        );
        return Err(AppError::InvalidRefreshToken);
    }

    if stored.expires_at <= Utc::now() {
        return Err(AppError::InvalidRefreshToken);
    }

    let user = repository::find_user_by_id(&mut *tx, stored.user_id)
        .await?
        .ok_or(AppError::InvalidRefreshToken)?;
    repository::revoke_refresh_token(&mut *tx, stored.id).await?;
    let session = issue_tokens(&mut tx, config, user, stored.family_id).await?;
    tx.commit().await?;

    Ok(session)
}

/// Revokes a refresh token. Unknown or already revoked tokens are ignored, so signing out
/// twice is not an error.
///
/// # Errors
///
/// Returns an error only if the database fails.
pub(super) async fn logout(db: &PgPool, refresh_token: &str) -> AppResult<()> {
    let token_hash = token::hash_refresh_token(refresh_token);
    repository::revoke_refresh_token_by_hash(db, &token_hash).await?;
    Ok(())
}

/// Loads the user behind an access token.
///
/// # Errors
///
/// Returns [`AppError::Unauthorized`] if the account no longer exists.
pub(super) async fn current_user(db: &PgPool, user_id: Uuid) -> AppResult<User> {
    repository::find_user_by_id(db, user_id)
        .await?
        .ok_or(AppError::Unauthorized)
}

/// Updates the signed-in user's account fields.
///
/// # Errors
///
/// Returns a validation error on `timezone` if PostgreSQL does not know the zone, and
/// [`AppError::Unauthorized`] if the user no longer exists.
pub(super) async fn update_current_user(
    db: &PgPool,
    user_id: Uuid,
    request: UpdateMeRequest,
) -> AppResult<User> {
    let timezone = match request.timezone.as_deref() {
        Some(name) => Some(
            repository::find_timezone_name(db, name.trim())
                .await?
                .ok_or_else(|| field_error("timezone", "timezone"))?,
        ),
        None => None,
    };

    let changes = UserChanges {
        display_name: request.display_name.as_deref().map(str::trim),
        timezone: timezone.as_deref(),
        locale: request.locale.map(Locale::as_str),
        currency: request.currency.as_deref(),
    };
    repository::update_user(db, user_id, changes)
        .await?
        .ok_or(AppError::Unauthorized)
}

async fn issue_tokens(
    conn: &mut PgConnection,
    config: &Config,
    user: User,
    family_id: Uuid,
) -> AppResult<Session> {
    let refresh = token::generate_refresh_token();
    let expires_at = Utc::now() + Duration::seconds(token::REFRESH_TOKEN_TTL_SECONDS);
    repository::insert_refresh_token(conn, user.id, &refresh.hash, family_id, expires_at).await?;
    let access_token = token::encode_access_token(&config.jwt_secret, user.id)?;

    Ok(Session {
        user,
        access_token,
        refresh_token: refresh.token,
    })
}

/// Value of `oauth_accounts.provider` for Google.
const GOOGLE_PROVIDER: &str = "google";

/// How a Google callback ended. The handler turns each case into a redirect to the frontend.
pub(super) enum GoogleCallback {
    /// The user is signed in; the session's refresh token goes into the cookie.
    SignedIn(Session),
    /// Google was linked to the account that started the flow.
    Linked,
    /// A sign-in flow failed.
    SignInFailed(AppError),
    /// A linking flow failed.
    LinkFailed(AppError),
}

/// Finishes a Google flow: signs the user in, or links Google to the account that started it.
pub(super) async fn google_callback(
    db: &PgPool,
    config: &Config,
    google: &GoogleClient,
    flow_token: Option<&str>,
    query: CallbackQuery,
) -> GoogleCallback {
    // Without a valid flow cookie there is no telling which kind of flow this was.
    let Some(flow) = flow_token.and_then(|token| google::decode_flow(&config.jwt_secret, token))
    else {
        return GoogleCallback::SignInFailed(AppError::GoogleSignInFailed);
    };

    match flow.link_user_id {
        Some(user_id) => match google_link(db, google, &flow, query, user_id).await {
            Ok(()) => GoogleCallback::Linked,
            Err(error) => GoogleCallback::LinkFailed(error),
        },
        None => match google_sign_in(db, config, google, &flow, query).await {
            Ok(session) => GoogleCallback::SignedIn(session),
            Err(error) => GoogleCallback::SignInFailed(error),
        },
    }
}

/// Signs in the account linked to the Google identity, creating it on first sign-in.
async fn google_sign_in(
    db: &PgPool,
    config: &Config,
    google: &GoogleClient,
    flow: &Flow,
    query: CallbackQuery,
) -> AppResult<Session> {
    let profile = google.complete(flow, query).await?;

    let mut tx = db.begin().await?;
    let existing =
        repository::find_user_by_oauth_account(&mut *tx, GOOGLE_PROVIDER, &profile.subject).await?;
    let user = match existing {
        Some(user) => user,
        None => create_google_user(&mut tx, &profile, flow.locale).await?,
    };
    let session = start_session(&mut tx, config, user).await?;
    tx.commit().await?;

    tracing::info!(user_id = %session.user.id, "user signed in with google");
    Ok(session)
}

/// Creates a password-less account for a Google identity seen for the first time.
///
/// An email that already belongs to a password account is **not** merged: that email was
/// never verified, so merging would let whoever registered it first take over the Google
/// user's data. The owner signs in with the password and links Google from Settings instead.
async fn create_google_user(
    conn: &mut PgConnection,
    profile: &GoogleProfile,
    locale: Option<Locale>,
) -> AppResult<User> {
    let display_name = profile
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| default_display_name(&profile.email));

    let user = create_user(
        &mut *conn,
        NewUser {
            email: &profile.email,
            password_hash: None,
            display_name,
            locale: locale.unwrap_or(Locale::En).as_str(),
        },
    )
    .await
    .map_err(|error| match error {
        AppError::EmailTaken => AppError::EmailAlreadyRegistered,
        other => other,
    })?;
    repository::insert_oauth_account(&mut *conn, user.id, GOOGLE_PROVIDER, &profile.subject)
        .await?;

    tracing::info!(user_id = %user.id, "user registered with google");
    Ok(user)
}

/// Links the Google identity to `user_id`. Linking the same pair again is a no-op.
async fn google_link(
    db: &PgPool,
    google: &GoogleClient,
    flow: &Flow,
    query: CallbackQuery,
    user_id: Uuid,
) -> AppResult<()> {
    let profile = google.complete(flow, query).await?;

    let owner =
        repository::find_user_by_oauth_account(db, GOOGLE_PROVIDER, &profile.subject).await?;
    match owner {
        Some(owner) if owner.id == user_id => Ok(()),
        Some(_) => Err(AppError::GoogleAccountAlreadyLinked),
        None => {
            repository::insert_oauth_account(db, user_id, GOOGLE_PROVIDER, &profile.subject)
                .await
                .map_err(|error| {
                    // The user already linked a different Google account.
                    if is_unique_violation(&error) {
                        AppError::GoogleAccountAlreadyLinked
                    } else {
                        AppError::Database(error)
                    }
                })?;
            tracing::info!(%user_id, "google account linked");
            Ok(())
        }
    }
}

/// The part of the email before `@`, used when the user gives no display name.
fn default_display_name(email: &str) -> &str {
    email.split('@').next().unwrap_or(email)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_display_name_is_the_email_local_part() {
        assert_eq!(default_display_name("ada@example.com"), "ada");
    }
}
