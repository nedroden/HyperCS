use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, patch, post},
};
use uuid::Uuid;

use crate::{
    auth::{AuthUser, OptionalAuthUser},
    dto::{BoardIndexResponse, BoardRequest, CategoryRequest, PageQuery, ReorderDirection},
    error::ApiResult,
    extract::ValidatedJson,
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
            get(board_detail).patch(update_board).delete(delete_board),
        )
        .route("/boards/{id}/reorder", post(reorder_board))
}

#[utoipa::path(get, path = "/api/v1/categories", responses((status = 200, body = BoardIndexResponse)), tag = "forum")]
async fn board_index(
    State(state): State<AppState>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> ApiResult<Json<BoardIndexResponse>> {
    let domain_user = user.map(|u| u.as_domain());
    let categories = structure_service::board_index(&state.db, domain_user.as_ref()).await?;
    Ok(Json(BoardIndexResponse { categories }))
}

#[utoipa::path(get, path = "/api/v1/boards/{id}", responses((status = 200)), tag = "forum")]
async fn board_detail(
    State(state): State<AppState>,
    OptionalAuthUser(user): OptionalAuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<PageQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let domain_user = user.map(|u| u.as_domain());
    let (board, topics) =
        structure_service::board_detail(&state.db, id, domain_user.as_ref(), q.page, q.per_page)
            .await?;
    Ok(Json(
        serde_json::json!({ "board": board, "topics": topics }),
    ))
}

#[utoipa::path(post, path = "/api/v1/categories", request_body = CategoryRequest, responses((status = 201)), tag = "admin")]
async fn create_category(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(req): ValidatedJson<CategoryRequest>,
) -> ApiResult<impl axum::response::IntoResponse> {
    let id = structure_service::create_category(&state.db, &user.as_domain(), &req).await?;
    Ok((
        StatusCode::CREATED,
        [("Location", format!("/api/v1/categories/{id}"))],
    ))
}

#[utoipa::path(patch, path = "/api/v1/categories/{id}", request_body = CategoryRequest, responses((status = 204)), tag = "admin")]
async fn update_category(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<CategoryRequest>,
) -> ApiResult<StatusCode> {
    structure_service::update_category(&state.db, &user.as_domain(), id, &req).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(delete, path = "/api/v1/categories/{id}", responses((status = 204)), tag = "admin")]
async fn delete_category(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    structure_service::delete_category(&state.db, &user.as_domain(), id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post, path = "/api/v1/categories/{id}/reorder", request_body = ReorderDirection, responses((status = 204)), tag = "admin")]
async fn reorder_category(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<ReorderDirection>,
) -> ApiResult<StatusCode> {
    structure_service::reorder_category(&state.db, &user.as_domain(), id, req.direction).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post, path = "/api/v1/boards", request_body = BoardRequest, responses((status = 201)), tag = "admin")]
async fn create_board(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(req): ValidatedJson<BoardRequest>,
) -> ApiResult<impl axum::response::IntoResponse> {
    let id = structure_service::create_board(&state.db, &user.as_domain(), &req).await?;
    Ok((
        StatusCode::CREATED,
        [("Location", format!("/api/v1/boards/{id}"))],
    ))
}

#[utoipa::path(patch, path = "/api/v1/boards/{id}", request_body = BoardRequest, responses((status = 204)), tag = "admin")]
async fn update_board(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<BoardRequest>,
) -> ApiResult<StatusCode> {
    structure_service::update_board(&state.db, &user.as_domain(), id, &req).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(delete, path = "/api/v1/boards/{id}", responses((status = 204)), tag = "admin")]
async fn delete_board(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    structure_service::delete_board(&state.db, &user.as_domain(), id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post, path = "/api/v1/boards/{id}/reorder", request_body = ReorderDirection, responses((status = 204)), tag = "admin")]
async fn reorder_board(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<ReorderDirection>,
) -> ApiResult<StatusCode> {
    structure_service::reorder_board(&state.db, &user.as_domain(), id, req.direction).await?;
    Ok(StatusCode::NO_CONTENT)
}
