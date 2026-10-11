use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, patch},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    dto::{
        ChangePasswordRequest, MeResponse, SessionInfo, UpdateAccountRequest,
        UpdatePreferencesRequest, UpdateProfileRequest,
    },
    error::{
        ApiResult, BadRequest, Forbidden, NotFound, PayloadTooLarge, Unauthorized, Unprocessable,
        UnsupportedMediaType,
    },
    extract::{ApiPath, ValidatedJson},
    services::user_service,
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(me))
        .route("/me/profile", patch(update_profile))
        .route("/me/account", patch(update_account))
        .route("/me/password", patch(change_password))
        .route("/me/preferences", patch(update_preferences))
        .route("/me/sessions", get(list_sessions))
        .route("/me/sessions/{id}", axum::routing::delete(revoke_session))
}

/// The logged-in account, including the CSRF token for this session.
#[utoipa::path(get, path = "/api/v1/me", operation_id = "me", tag = "me",
    security(("session_cookie" = [])),
    responses((status = 200, body = MeResponse), Unauthorized))]
async fn me(State(state): State<AppState>, user: AuthUser) -> ApiResult<Json<MeResponse>> {
    Ok(Json(user_service::get_me(&state.db, &user).await?))
}

/// Partially update the public profile (absent = unchanged, `null` = clear).
#[utoipa::path(patch, path = "/api/v1/me/profile", operation_id = "update_profile", tag = "me",
    security(("session_cookie" = [], "csrf_token" = [])),
    request_body = UpdateProfileRequest,
    responses((status = 204, description = "Updated."),
        BadRequest, Unauthorized, Forbidden, PayloadTooLarge, UnsupportedMediaType, Unprocessable))]
async fn update_profile(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(req): ValidatedJson<UpdateProfileRequest>,
) -> ApiResult<StatusCode> {
    user_service::update_profile(&state.db, user.id, &req).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Partially update the private account data (name, email).
#[utoipa::path(patch, path = "/api/v1/me/account", operation_id = "update_account", tag = "me",
    security(("session_cookie" = [], "csrf_token" = [])),
    request_body = UpdateAccountRequest,
    responses((status = 204, description = "Updated."),
        BadRequest, Unauthorized, Forbidden, PayloadTooLarge, UnsupportedMediaType, Unprocessable))]
async fn update_account(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(req): ValidatedJson<UpdateAccountRequest>,
) -> ApiResult<StatusCode> {
    user_service::update_account(&state.db, user.id, &req).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Change the password. All other sessions of the account are revoked.
#[utoipa::path(patch, path = "/api/v1/me/password", operation_id = "change_password", tag = "me",
    security(("session_cookie" = [], "csrf_token" = [])),
    request_body = ChangePasswordRequest,
    responses((status = 204, description = "Changed; other sessions revoked."),
        BadRequest, Unauthorized, Forbidden, PayloadTooLarge, UnsupportedMediaType, Unprocessable))]
async fn change_password(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(req): ValidatedJson<ChangePasswordRequest>,
) -> ApiResult<StatusCode> {
    user_service::change_password(&state.db, &user, &req).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Partially update preferences (language, time zone, notifications, privacy).
#[utoipa::path(patch, path = "/api/v1/me/preferences", operation_id = "update_preferences", tag = "me",
    security(("session_cookie" = [], "csrf_token" = [])),
    request_body = UpdatePreferencesRequest,
    responses((status = 204, description = "Updated."),
        BadRequest, Unauthorized, Forbidden, PayloadTooLarge, UnsupportedMediaType, Unprocessable))]
async fn update_preferences(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(req): ValidatedJson<UpdatePreferencesRequest>,
) -> ApiResult<StatusCode> {
    user_service::update_preferences(&state.db, user.id, &req).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The account's active sessions (not paginated: bounded by the number of
/// devices a person signs in from).
#[utoipa::path(get, path = "/api/v1/me/sessions", operation_id = "list_sessions", tag = "me",
    security(("session_cookie" = [])),
    responses((status = 200, body = Vec<SessionInfo>), Unauthorized))]
async fn list_sessions(
    State(state): State<AppState>,
    user: AuthUser,
) -> ApiResult<Json<Vec<SessionInfo>>> {
    Ok(Json(user_service::list_sessions(&state.db, &user).await?))
}

/// Revoke one of the caller's own sessions.
#[utoipa::path(delete, path = "/api/v1/me/sessions/{id}", operation_id = "revoke_session", tag = "me",
    security(("session_cookie" = [], "csrf_token" = [])),
    params(("id" = Uuid, Path, description = "Session id")),
    responses((status = 204, description = "Revoked."), Unauthorized, Forbidden, NotFound))]
async fn revoke_session(
    State(state): State<AppState>,
    user: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
) -> ApiResult<StatusCode> {
    user_service::revoke_session(&state.db, user.id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
