use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use axum_extra::extract::CookieJar;
use hypercs_domain::constants::AUTH_BODY_LIMIT_BYTES;

use crate::{
    auth::{
        AuthUser, SameOrigin,
        cookie::{build_logout_cookie, build_session_cookie},
        extractor::SESSION_COOKIE,
    },
    dto::{LoginRequest, RegisterRequest, UserSummary},
    error::{
        ApiResult, BadRequest, Forbidden, PayloadTooLarge, Unauthorized, Unprocessable,
        UnsupportedMediaType,
    },
    extract::ValidatedJson,
    services::auth_service,
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    let limit = DefaultBodyLimit::max(AUTH_BODY_LIMIT_BYTES);
    Router::new()
        .route("/auth/register", post(register).layer(limit))
        .route("/auth/login", post(login).layer(limit))
        .route("/auth/logout", post(logout))
}

/// Create a member account. Does not log the user in.
#[utoipa::path(post, path = "/api/v1/auth/register", operation_id = "register", tag = "auth",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Account created.", body = UserSummary,
            headers(("Location" = String, description = "URL of the new user's public profile"))),
        BadRequest, Forbidden, PayloadTooLarge, UnsupportedMediaType,
        Unprocessable,
    ))]
async fn register(
    State(state): State<AppState>,
    _origin: SameOrigin,
    ValidatedJson(req): ValidatedJson<RegisterRequest>,
) -> ApiResult<Response> {
    let user = auth_service::register(&state.db, &req).await?;
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, format!("/api/v1/users/{}", user.username))],
        Json::<UserSummary>(user),
    )
        .into_response())
}

/// Log in with username and password. Sets the HTTP-only `hypercs_session`
/// cookie (any previous session cookie is revoked: the session is rotated).
/// Read the CSRF token from `GET /me` afterwards.
#[utoipa::path(post, path = "/api/v1/auth/login", operation_id = "login", tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 204, description = "Logged in; session cookie set.",
            headers(("Set-Cookie" = String, description = "`hypercs_session` (HttpOnly, SameSite=Lax, Secure in production)"))),
        BadRequest, Unauthorized, Forbidden, PayloadTooLarge, UnsupportedMediaType,
        Unprocessable,
    ))]
async fn login(
    State(state): State<AppState>,
    _origin: SameOrigin,
    jar: CookieJar,
    headers: HeaderMap,
    ValidatedJson(req): ValidatedJson<LoginRequest>,
) -> ApiResult<impl IntoResponse> {
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok());
    let previous = jar.get(SESSION_COOKIE).map(|c| c.value().to_string());
    let session = auth_service::login(&state.db, &req, user_agent, previous.as_deref()).await?;
    let cookie = build_session_cookie(session.token, session.remember, state.config.cookie_secure);
    Ok((StatusCode::NO_CONTENT, jar.add(cookie)))
}

/// Log out: deletes the current session.
#[utoipa::path(post, path = "/api/v1/auth/logout", operation_id = "logout", tag = "auth",
    security(("session_cookie" = [], "csrf_token" = [])),
    responses(
        (status = 204, description = "Logged out; session cookie cleared."),
        Unauthorized, Forbidden,
    ))]
async fn logout(
    State(state): State<AppState>,
    user: AuthUser,
    jar: CookieJar,
) -> ApiResult<impl IntoResponse> {
    auth_service::logout(&state.db, user.session_id).await?;
    let cookie = build_logout_cookie(state.config.cookie_secure);
    Ok((StatusCode::NO_CONTENT, jar.add(cookie)))
}
