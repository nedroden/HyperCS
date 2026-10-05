use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, patch},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    dto::{
        ChangePasswordRequest, MeResponse, UpdateAccountRequest, UpdatePreferencesRequest,
        UpdateProfileRequest,
    },
    error::{ApiError, ApiResult},
    extract::ValidatedJson,
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

#[utoipa::path(get, path = "/api/v1/me", responses((status = 200, body = MeResponse)), tag = "me")]
async fn me(State(state): State<AppState>, user: AuthUser) -> ApiResult<Json<MeResponse>> {
    let row = user_service::get_me(&state.db, user.id).await?;
    let csrf_token = hypercs_db::sessions::csrf_token_for_session(&state.db, user.session_id)
        .await
        .map_err(ApiError::Sqlx)?
        .ok_or(ApiError::Domain(hypercs_domain::DomainError::Unauthorized))?;
    Ok(Json(MeResponse {
        id: row.id,
        username: row.username,
        display_name: row.display_name,
        email: row.email,
        role: row.role,
        first_name: row.first_name,
        last_name: row.last_name,
        location: row.location,
        website: row.website,
        about_me: row.about_me,
        language: row.language,
        timezone: row.timezone,
        notify_on_reply: row.notify_on_reply,
        notify_on_quote: row.notify_on_quote,
        show_online: row.show_online,
        post_count: row.post_count,
        created_at: row.created_at,
        csrf_token,
    }))
}

#[utoipa::path(patch, path = "/api/v1/me/profile", request_body = UpdateProfileRequest,
    responses((status = 204)), tag = "me")]
async fn update_profile(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(req): ValidatedJson<UpdateProfileRequest>,
) -> ApiResult<StatusCode> {
    user_service::update_profile(&state.db, user.id, &req).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(patch, path = "/api/v1/me/account", request_body = UpdateAccountRequest,
    responses((status = 204)), tag = "me")]
async fn update_account(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(req): ValidatedJson<UpdateAccountRequest>,
) -> ApiResult<StatusCode> {
    user_service::update_account(&state.db, user.id, &req).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(patch, path = "/api/v1/me/password", request_body = ChangePasswordRequest,
    responses((status = 204)), tag = "me")]
async fn change_password(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(req): ValidatedJson<ChangePasswordRequest>,
) -> ApiResult<StatusCode> {
    user_service::change_password(&state.db, user.id, user.session_id, &req).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(patch, path = "/api/v1/me/preferences", request_body = UpdatePreferencesRequest,
    responses((status = 204)), tag = "me")]
async fn update_preferences(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(req): ValidatedJson<UpdatePreferencesRequest>,
) -> ApiResult<StatusCode> {
    user_service::update_preferences(&state.db, user.id, &req).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/api/v1/me/sessions", responses((status = 200)), tag = "me")]
async fn list_sessions(
    State(state): State<AppState>,
    user: AuthUser,
) -> ApiResult<Json<Vec<hypercs_db::sessions::SessionListRow>>> {
    let sessions = user_service::list_sessions(&state.db, user.id).await?;
    Ok(Json(sessions))
}

#[utoipa::path(delete, path = "/api/v1/me/sessions/{id}", responses((status = 204)), tag = "me")]
async fn revoke_session(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    user_service::revoke_session(&state.db, user.id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
