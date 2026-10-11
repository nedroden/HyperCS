use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, patch},
};

use crate::{
    auth::AuthUser,
    dto::{AdminUserPage, PageQuery, PublicProfile, UpdateRoleRequest},
    error::{
        ApiResult, BadRequest, Conflict, Forbidden, NotFound, PayloadTooLarge, Unauthorized,
        Unprocessable, UnsupportedMediaType,
    },
    extract::{ApiPath, ApiQuery, ValidatedJson},
    services::user_service,
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users))
        .route("/users/{username}", get(public_profile))
        .route("/users/{username}/role", patch(update_role))
}

/// A member's public profile (looked up by case-insensitive username).
#[utoipa::path(get, path = "/api/v1/users/{username}", operation_id = "public_profile", tag = "users",
    params(("username" = String, Path, description = "Username (case-insensitive)")),
    responses((status = 200, body = PublicProfile), NotFound))]
async fn public_profile(
    State(state): State<AppState>,
    ApiPath(username): ApiPath<String>,
) -> ApiResult<Json<PublicProfile>> {
    Ok(Json(
        user_service::public_profile(&state.db, &username).await?,
    ))
}

/// Admin: all accounts, oldest first.
#[utoipa::path(get, path = "/api/v1/users", operation_id = "list_users", tag = "admin",
    security(("session_cookie" = [])),
    params(PageQuery),
    responses((status = 200, body = AdminUserPage), BadRequest, Unauthorized, Forbidden))]
async fn list_users(
    State(state): State<AppState>,
    user: AuthUser,
    ApiQuery(q): ApiQuery<PageQuery>,
) -> ApiResult<Json<AdminUserPage>> {
    Ok(Json(
        user_service::list_users(&state.db, &user.as_domain(), q.page, q.per_page).await?,
    ))
}

/// Admin: assign a role. Applies immediately (roles are read from the
/// database on every request). The last admin cannot be demoted.
#[utoipa::path(patch, path = "/api/v1/users/{username}/role", operation_id = "update_role", tag = "admin",
    security(("session_cookie" = [], "csrf_token" = [])),
    params(("username" = String, Path, description = "Username (case-insensitive)")),
    request_body = UpdateRoleRequest,
    responses((status = 204, description = "Role changed."),
        BadRequest, Unauthorized, Forbidden, NotFound, Conflict, PayloadTooLarge,
        UnsupportedMediaType, Unprocessable))]
async fn update_role(
    State(state): State<AppState>,
    user: AuthUser,
    ApiPath(username): ApiPath<String>,
    ValidatedJson(req): ValidatedJson<UpdateRoleRequest>,
) -> ApiResult<StatusCode> {
    user_service::update_role(&state.db, &user.as_domain(), &username, req.role).await?;
    Ok(StatusCode::NO_CONTENT)
}
