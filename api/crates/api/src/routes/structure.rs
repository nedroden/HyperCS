use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::Response,
    routing::{get, patch, post},
};
use uuid::Uuid;

use super::created;
use crate::{
    auth::{AuthUser, OptionalAuthUser},
    dto::{
        BoardIndexResponse, BoardPageResponse, BoardRequest, CategoryRequest, CreatedResponse,
        PageQuery, ReorderRequest, UpdateBoardRequest, UpdateCategoryRequest,
    },
    error::{
        ApiResult, BadRequest, Forbidden, NotFound, PayloadTooLarge, Unauthorized, Unprocessable,
        UnsupportedMediaType,
    },
    extract::{ApiPath, ApiQuery, ValidatedJson},
    services::structure_service,
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/categories", get(board_index).post(create_category))
        .route(
            "/categories/{id}",
            patch(update_category).delete(delete_category),
        )
        .route("/categories/{id}/reorder", post(reorder_category))
        .route("/boards", post(create_board))
        .route(
            "/boards/{id}",
            get(board_page).patch(update_board).delete(delete_board),
        )
        .route("/boards/{id}/reorder", post(reorder_board))
}

/// The board index: categories with their boards, counters and last posts.
/// Admin-only categories/boards are included for admins only.
#[utoipa::path(get, path = "/api/v1/categories", operation_id = "board_index", tag = "forum",
    security((), ("session_cookie" = [])),
    responses((status = 200, body = BoardIndexResponse)))]
async fn board_index(
    State(state): State<AppState>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> ApiResult<Json<BoardIndexResponse>> {
    let viewer = user.map(|u| u.as_domain());
    Ok(Json(
        structure_service::board_index(&state.db, viewer.as_ref()).await?,
    ))
}

/// A board with one page of its topics (pinned first, then latest activity).
/// Admin-only boards are `404` for everyone but admins.
#[utoipa::path(get, path = "/api/v1/boards/{id}", operation_id = "board_page", tag = "forum",
    security((), ("session_cookie" = [])),
    params(("id" = Uuid, Path, description = "Board id"), PageQuery),
    responses((status = 200, body = BoardPageResponse), BadRequest, NotFound))]
async fn board_page(
    State(state): State<AppState>,
    OptionalAuthUser(user): OptionalAuthUser,
    ApiPath(id): ApiPath<Uuid>,
    ApiQuery(q): ApiQuery<PageQuery>,
) -> ApiResult<Json<BoardPageResponse>> {
    let viewer = user.map(|u| u.as_domain());
    Ok(Json(
        structure_service::board_page(&state.db, id, viewer.as_ref(), q.page, q.per_page).await?,
    ))
}

/// Admin: create a category (appended at the end).
#[utoipa::path(post, path = "/api/v1/categories", operation_id = "create_category", tag = "admin",
    security(("session_cookie" = [], "csrf_token" = [])),
    request_body = CategoryRequest,
    responses((status = 201, description = "Created.", body = CreatedResponse,
            headers(("Location" = String, description = "URL of the new category"))),
        BadRequest, Unauthorized, Forbidden, PayloadTooLarge, UnsupportedMediaType, Unprocessable))]
async fn create_category(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(req): ValidatedJson<CategoryRequest>,
) -> ApiResult<Response> {
    let id = structure_service::create_category(&state.db, &user.as_domain(), &req).await?;
    Ok(created(format!("/api/v1/categories/{id}"), id))
}

/// Admin: partially update a category.
#[utoipa::path(patch, path = "/api/v1/categories/{id}", operation_id = "update_category", tag = "admin",
    security(("session_cookie" = [], "csrf_token" = [])),
    params(("id" = Uuid, Path, description = "Category id")),
    request_body = UpdateCategoryRequest,
    responses((status = 204, description = "Updated."),
        BadRequest, Unauthorized, Forbidden, NotFound, PayloadTooLarge, UnsupportedMediaType,
        Unprocessable))]
async fn update_category(
    State(state): State<AppState>,
    user: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
    ValidatedJson(req): ValidatedJson<UpdateCategoryRequest>,
) -> ApiResult<StatusCode> {
    structure_service::update_category(&state.db, &user.as_domain(), id, &req).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Admin: hard-delete a category with all its boards, topics and posts.
#[utoipa::path(delete, path = "/api/v1/categories/{id}", operation_id = "delete_category", tag = "admin",
    security(("session_cookie" = [], "csrf_token" = [])),
    params(("id" = Uuid, Path, description = "Category id")),
    responses((status = 204, description = "Deleted (cascade)."), Unauthorized, Forbidden, NotFound))]
async fn delete_category(
    State(state): State<AppState>,
    user: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
) -> ApiResult<StatusCode> {
    structure_service::delete_category(&state.db, &user.as_domain(), id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Admin: move a category one position up or down (a no-op at the edge).
#[utoipa::path(post, path = "/api/v1/categories/{id}/reorder", operation_id = "reorder_category", tag = "admin",
    security(("session_cookie" = [], "csrf_token" = [])),
    params(("id" = Uuid, Path, description = "Category id")),
    request_body = ReorderRequest,
    responses((status = 204, description = "Reordered."),
        BadRequest, Unauthorized, Forbidden, NotFound, PayloadTooLarge, UnsupportedMediaType,
        Unprocessable))]
async fn reorder_category(
    State(state): State<AppState>,
    user: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
    ValidatedJson(req): ValidatedJson<ReorderRequest>,
) -> ApiResult<StatusCode> {
    structure_service::reorder_category(&state.db, &user.as_domain(), id, req.direction).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Admin: create a board in a category (appended at the end).
#[utoipa::path(post, path = "/api/v1/boards", operation_id = "create_board", tag = "admin",
    security(("session_cookie" = [], "csrf_token" = [])),
    request_body = BoardRequest,
    responses((status = 201, description = "Created.", body = CreatedResponse,
            headers(("Location" = String, description = "URL of the new board"))),
        BadRequest, Unauthorized, Forbidden, PayloadTooLarge, UnsupportedMediaType, Unprocessable))]
async fn create_board(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(req): ValidatedJson<BoardRequest>,
) -> ApiResult<Response> {
    let id = structure_service::create_board(&state.db, &user.as_domain(), &req).await?;
    Ok(created(format!("/api/v1/boards/{id}"), id))
}

/// Admin: partially update a board.
#[utoipa::path(patch, path = "/api/v1/boards/{id}", operation_id = "update_board", tag = "admin",
    security(("session_cookie" = [], "csrf_token" = [])),
    params(("id" = Uuid, Path, description = "Board id")),
    request_body = UpdateBoardRequest,
    responses((status = 204, description = "Updated."),
        BadRequest, Unauthorized, Forbidden, NotFound, PayloadTooLarge, UnsupportedMediaType,
        Unprocessable))]
async fn update_board(
    State(state): State<AppState>,
    user: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
    ValidatedJson(req): ValidatedJson<UpdateBoardRequest>,
) -> ApiResult<StatusCode> {
    structure_service::update_board(&state.db, &user.as_domain(), id, &req).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Admin: hard-delete a board with its topics and posts.
#[utoipa::path(delete, path = "/api/v1/boards/{id}", operation_id = "delete_board", tag = "admin",
    security(("session_cookie" = [], "csrf_token" = [])),
    params(("id" = Uuid, Path, description = "Board id")),
    responses((status = 204, description = "Deleted (cascade)."), Unauthorized, Forbidden, NotFound))]
async fn delete_board(
    State(state): State<AppState>,
    user: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
) -> ApiResult<StatusCode> {
    structure_service::delete_board(&state.db, &user.as_domain(), id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Admin: move a board one position up or down within its category.
#[utoipa::path(post, path = "/api/v1/boards/{id}/reorder", operation_id = "reorder_board", tag = "admin",
    security(("session_cookie" = [], "csrf_token" = [])),
    params(("id" = Uuid, Path, description = "Board id")),
    request_body = ReorderRequest,
    responses((status = 204, description = "Reordered."),
        BadRequest, Unauthorized, Forbidden, NotFound, PayloadTooLarge, UnsupportedMediaType,
        Unprocessable))]
async fn reorder_board(
    State(state): State<AppState>,
    user: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
    ValidatedJson(req): ValidatedJson<ReorderRequest>,
) -> ApiResult<StatusCode> {
    structure_service::reorder_board(&state.db, &user.as_domain(), id, req.direction).await?;
    Ok(StatusCode::NO_CONTENT)
}
