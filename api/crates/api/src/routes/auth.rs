use axum::{
    Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
};
use axum_extra::extract::CookieJar;

use crate::{
    auth::{
        AuthUser, SameOrigin,
        cookie::{build_logout_cookie, build_session_cookie},
    },
    dto::{LoginRequest, RegisterRequest},
    error::ApiResult,
    extract::ValidatedJson,
    services::auth_service,
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
}

#[utoipa::path(post, path = "/api/v1/auth/register", request_body = RegisterRequest,
    responses((status = 201, description = "Registered"), (status = 422, description = "Validation failed")),
    tag = "auth")]
async fn register(
    State(state): State<AppState>,
    _origin: SameOrigin,
    ValidatedJson(req): ValidatedJson<RegisterRequest>,
) -> ApiResult<impl IntoResponse> {
    let user = auth_service::register(&state.db, &req).await?;
    Ok((
        StatusCode::CREATED,
        [("Location", format!("/api/v1/users/{}", user.id))],
    ))
}

#[utoipa::path(post, path = "/api/v1/auth/login", request_body = LoginRequest,
    responses((status = 204, description = "Logged in"), (status = 422, description = "Invalid credentials")),
    tag = "auth")]
async fn login(
    State(state): State<AppState>,
    _origin: SameOrigin,
    jar: CookieJar,
    headers: HeaderMap,
    ValidatedJson(req): ValidatedJson<LoginRequest>,
) -> ApiResult<impl IntoResponse> {
    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok());
    let session = auth_service::login(&state.db, &req, user_agent, None).await?;
    let cookie = build_session_cookie(session.token, session.remember, state.config.cookie_secure);
    Ok((StatusCode::NO_CONTENT, jar.add(cookie)))
}

#[utoipa::path(post, path = "/api/v1/auth/logout", responses((status = 204, description = "Logged out")), tag = "auth")]
async fn logout(
    State(state): State<AppState>,
    user: AuthUser,
    jar: CookieJar,
) -> ApiResult<impl IntoResponse> {
    auth_service::logout(&state.db, user.session_id).await?;
    let cookie = build_logout_cookie(state.config.cookie_secure);
    Ok((StatusCode::NO_CONTENT, jar.add(cookie)))
}
