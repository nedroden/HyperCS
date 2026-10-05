use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, patch},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    dto::UpdateRoleRequest,
    error::{ApiError, ApiResult},
    extract::ValidatedJson,
    services::user_service,
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users))
        .route("/users/{username}", get(public_profile))
        .route("/users/by-id/{id}/role", patch(update_role))
}

#[utoipa::path(get, path = "/api/v1/users/{username}", responses((status = 200)), tag = "users")]
async fn public_profile(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> ApiResult<Json<hypercs_db::users::PublicProfileRow>> {
    let profile = user_service::public_profile(&state.db, &username).await?;
    Ok(Json(profile))
}

#[utoipa::path(get, path = "/api/v1/users", responses((status = 200)), tag = "users")]
async fn list_users(
    State(state): State<AppState>,
    user: AuthUser,
) -> ApiResult<Json<Vec<hypercs_db::users::AdminUserRow>>> {
    if !hypercs_domain::policy::can_manage_users(&user.as_domain()) {
        return Err(ApiError::Domain(hypercs_domain::DomainError::Forbidden));
    }
    let users = user_service::list_all_users(&state.db).await?;
    Ok(Json(users))
}

#[utoipa::path(patch, path = "/api/v1/users/by-id/{id}/role", request_body = UpdateRoleRequest,
    responses((status = 204)), tag = "users")]
async fn update_role(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<UpdateRoleRequest>,
) -> ApiResult<StatusCode> {
    if !hypercs_domain::policy::can_manage_users(&user.as_domain()) {
        return Err(ApiError::Domain(hypercs_domain::DomainError::Forbidden));
    }
    user_service::update_role(&state.db, id, &req.role).await?;
    Ok(StatusCode::NO_CONTENT)
}
