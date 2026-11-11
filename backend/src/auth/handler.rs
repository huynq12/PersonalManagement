//! HTTP handlers of the auth feature: extraction, cookies, and response shaping only.

use axum::{
    Json,
    extract::{Query, State, rejection::QueryRejection},
    http::StatusCode,
    response::Redirect,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};

use super::{
    extractor::AuthUser,
    google::{self, CallbackQuery},
    model::{
        GoogleLinkResponse, GoogleStartQuery, LoginRequest, RegisterRequest, SessionResponse,
        UserResponse,
    },
    service::{self, GoogleCallback, Session},
    token,
};
use crate::{
    AppState,
    config::Config,
    error::{AppError, AppResult},
    validation::ValidatedJson,
};

/// Name of the cookie that carries the refresh token.
pub(super) const REFRESH_COOKIE: &str = "refresh_token";
/// The refresh cookie is only sent to the auth routes, never with ordinary API calls.
pub(super) const AUTH_COOKIE_PATH: &str = "/api/v1/auth";
/// Name of the cookie that carries the signed OAuth flow token during a Google round trip.
const FLOW_COOKIE: &str = "oauth_flow";
/// The flow cookie is only sent to the Google routes.
const FLOW_COOKIE_PATH: &str = "/api/v1/auth/google";

/// `POST /api/v1/auth/register` — creates an account without signing in.
pub(super) async fn register(
    State(state): State<AppState>,
    ValidatedJson(request): ValidatedJson<RegisterRequest>,
) -> AppResult<(StatusCode, Json<UserResponse>)> {
    let user = service::register(&state.db, request).await?;
    Ok((StatusCode::CREATED, Json(user.into())))
}

/// `POST /api/v1/auth/login` — returns an access token and sets the refresh cookie.
pub(super) async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    ValidatedJson(request): ValidatedJson<LoginRequest>,
) -> AppResult<(CookieJar, Json<SessionResponse>)> {
    let session = service::login(&state.db, &state.config, request).await?;
    Ok(session_response(jar, &state.config, session))
}

/// `POST /api/v1/auth/refresh` — rotates the refresh cookie and returns a new access token.
pub(super) async fn refresh(
    State(state): State<AppState>,
    jar: CookieJar,
) -> AppResult<(CookieJar, Json<SessionResponse>)> {
    let refresh_token = jar
        .get(REFRESH_COOKIE)
        .map(Cookie::value)
        .ok_or(AppError::InvalidRefreshToken)?;
    let session = service::refresh(&state.db, &state.config, refresh_token).await?;
    Ok(session_response(jar, &state.config, session))
}

/// `POST /api/v1/auth/logout` — revokes the refresh token and clears the cookie.
pub(super) async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> AppResult<(CookieJar, StatusCode)> {
    if let Some(cookie) = jar.get(REFRESH_COOKIE) {
        service::logout(&state.db, cookie.value()).await?;
    }
    // Removal only works when the path matches the one the cookie was set with.
    let jar = jar.remove(Cookie::build(REFRESH_COOKIE).path(AUTH_COOKIE_PATH));
    Ok((jar, StatusCode::NO_CONTENT))
}

/// `GET /api/v1/me` — the signed-in user.
pub(super) async fn me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<UserResponse>> {
    let user = service::current_user(&state.db, auth.id).await?;
    Ok(Json(user.into()))
}

/// `GET /api/v1/auth/google` — sends the browser to Google to sign in.
pub(super) async fn google_start(
    State(state): State<AppState>,
    jar: CookieJar,
    query: Result<Query<GoogleStartQuery>, QueryRejection>,
) -> AppResult<(CookieJar, Redirect)> {
    let google = state.google.as_deref().ok_or(AppError::NotFound)?;
    let Query(query) = query.map_err(|_| AppError::InvalidRequest)?;

    let started = google.start(&state.config.jwt_secret, None, query.locale)?;
    let jar = jar.add(flow_cookie(&state.config, started.flow_token));
    Ok((jar, Redirect::to(&started.authorization_url)))
}

/// `POST /api/v1/auth/google/link` — starts linking Google to the signed-in account.
///
/// Returns the Google URL instead of redirecting: the request is a `fetch` carrying the access
/// token, and the browser must then navigate to the URL itself.
pub(super) async fn google_link(
    State(state): State<AppState>,
    auth: AuthUser,
    jar: CookieJar,
) -> AppResult<(CookieJar, Json<GoogleLinkResponse>)> {
    let google = state.google.as_deref().ok_or(AppError::NotFound)?;

    let started = google.start(&state.config.jwt_secret, Some(auth.id), None)?;
    let jar = jar.add(flow_cookie(&state.config, started.flow_token));
    let body = GoogleLinkResponse {
        authorization_url: started.authorization_url,
    };
    Ok((jar, Json(body)))
}

/// `GET /api/v1/auth/google/callback` — where Google sends the browser back.
///
/// This is a page navigation, not an API call, so every outcome is a redirect to the frontend.
/// On success the refresh cookie is set and the frontend calls `/auth/refresh` to get an
/// access token; failures carry an error `code` for the frontend to translate.
pub(super) async fn google_callback(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<CallbackQuery>,
) -> AppResult<(CookieJar, Redirect)> {
    let google = state.google.as_deref().ok_or(AppError::NotFound)?;
    let flow_token = jar.get(FLOW_COOKIE).map(|cookie| cookie.value().to_owned());

    let outcome = service::google_callback(
        &state.db,
        &state.config,
        google,
        flow_token.as_deref(),
        query,
    )
    .await;

    // The flow token is single-use from the browser's point of view.
    let jar = jar.remove(Cookie::build(FLOW_COOKIE).path(FLOW_COOKIE_PATH));
    let frontend = &state.config.cors_origin;
    let (jar, target) = match outcome {
        GoogleCallback::SignedIn(session) => (
            jar.add(refresh_cookie(&state.config, session.refresh_token)),
            format!("{frontend}/auth/callback"),
        ),
        GoogleCallback::Linked => (jar, format!("{frontend}/settings?google=linked")),
        GoogleCallback::SignInFailed(error) => {
            error.log_if_unexpected();
            (jar, format!("{frontend}/login?error={}", error.code()))
        }
        GoogleCallback::LinkFailed(error) => {
            error.log_if_unexpected();
            (jar, format!("{frontend}/settings?error={}", error.code()))
        }
    };

    Ok((jar, Redirect::to(&target)))
}

/// Builds the refresh cookie.
///
/// `HttpOnly` keeps the token away from JavaScript. Behind HTTPS the frontend and the API live
/// on different sites, so the cookie must be `SameSite=None; Secure` to be sent at all; in
/// local development both run on `localhost`, where `Lax` is enough.
fn refresh_cookie(config: &Config, refresh_token: String) -> Cookie<'static> {
    Cookie::build((REFRESH_COOKIE, refresh_token))
        .http_only(true)
        .secure(config.cookie_secure)
        .same_site(same_site(config))
        .path(AUTH_COOKIE_PATH)
        .max_age(time::Duration::seconds(token::REFRESH_TOKEN_TTL_SECONDS))
        .build()
}

/// Builds the cookie that carries the OAuth flow token to the Google callback.
fn flow_cookie(config: &Config, flow_token: String) -> Cookie<'static> {
    Cookie::build((FLOW_COOKIE, flow_token))
        .http_only(true)
        .secure(config.cookie_secure)
        .same_site(same_site(config))
        .path(FLOW_COOKIE_PATH)
        .max_age(time::Duration::seconds(google::FLOW_TTL_SECONDS))
        .build()
}

fn same_site(config: &Config) -> SameSite {
    if config.cookie_secure {
        SameSite::None
    } else {
        SameSite::Lax
    }
}

fn session_response(
    jar: CookieJar,
    config: &Config,
    session: Session,
) -> (CookieJar, Json<SessionResponse>) {
    let jar = jar.add(refresh_cookie(config, session.refresh_token));
    let body = SessionResponse {
        access_token: session.access_token,
        token_type: "Bearer",
        expires_in: token::ACCESS_TOKEN_TTL_SECONDS,
        user: session.user.into(),
    };
    (jar, Json(body))
}
