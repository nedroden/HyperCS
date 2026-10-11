use axum::{Json, Router, extract::State, http::StatusCode, response::Response, routing::get};
use uuid::Uuid;

use super::created;
use crate::{
    auth::AuthUser,
    dto::{
        CreateNewsRequest, CreatedResponse, NewsItem, NewsItemPage, PageQuery, UpdateNewsRequest,
    },
    error::{
        ApiResult, BadRequest, Forbidden, NotFound, PayloadTooLarge, Unauthorized, Unprocessable,
        UnsupportedMediaType,
    },
    extract::{ApiPath, ApiQuery, ValidatedJson},
    services::news_service,
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/news", get(list_news).post(create_news))
        .route(
            "/news/{id}",
            get(get_news).patch(update_news).delete(delete_news),
        )
}

/// News items, newest first.
#[utoipa::path(get, path = "/api/v1/news", operation_id = "list_news", tag = "cms",
    params(PageQuery),
    responses((status = 200, body = NewsItemPage), BadRequest))]
async fn list_news(
    State(state): State<AppState>,
    ApiQuery(q): ApiQuery<PageQuery>,
) -> ApiResult<Json<NewsItemPage>> {
    Ok(Json(
        news_service::list(&state.db, q.page, q.per_page).await?,
    ))
}

/// One news item, addressed by id (the slug is cosmetic).
#[utoipa::path(get, path = "/api/v1/news/{id}", operation_id = "get_news", tag = "cms",
    params(("id" = Uuid, Path, description = "News item id")),
    responses((status = 200, body = NewsItem), NotFound))]
async fn get_news(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> ApiResult<Json<NewsItem>> {
    Ok(Json(news_service::get(&state.db, id).await?))
}

/// Admin: publish a news item.
#[utoipa::path(post, path = "/api/v1/news", operation_id = "create_news", tag = "admin",
    security(("session_cookie" = [], "csrf_token" = [])),
    request_body = CreateNewsRequest,
    responses((status = 201, description = "Created.", body = CreatedResponse,
            headers(("Location" = String, description = "URL of the new news item"))),
        BadRequest, Unauthorized, Forbidden, PayloadTooLarge, UnsupportedMediaType, Unprocessable))]
async fn create_news(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(req): ValidatedJson<CreateNewsRequest>,
) -> ApiResult<Response> {
    let id = news_service::create(&state.db, &user.as_domain(), &req).await?;
    Ok(created(format!("/api/v1/news/{id}"), id))
}

/// Admin: partially update a news item.
#[utoipa::path(patch, path = "/api/v1/news/{id}", operation_id = "update_news", tag = "admin",
    security(("session_cookie" = [], "csrf_token" = [])),
    params(("id" = Uuid, Path, description = "News item id")),
    request_body = UpdateNewsRequest,
    responses((status = 204, description = "Updated."),
        BadRequest, Unauthorized, Forbidden, NotFound, PayloadTooLarge, UnsupportedMediaType,
        Unprocessable))]
async fn update_news(
    State(state): State<AppState>,
    user: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
    ValidatedJson(req): ValidatedJson<UpdateNewsRequest>,
) -> ApiResult<StatusCode> {
    news_service::update(&state.db, &user.as_domain(), id, &req).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Admin: delete a news item.
#[utoipa::path(delete, path = "/api/v1/news/{id}", operation_id = "delete_news", tag = "admin",
    security(("session_cookie" = [], "csrf_token" = [])),
    params(("id" = Uuid, Path, description = "News item id")),
    responses((status = 204, description = "Deleted."), Unauthorized, Forbidden, NotFound))]
async fn delete_news(
    State(state): State<AppState>,
    user: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
) -> ApiResult<StatusCode> {
    news_service::delete(&state.db, &user.as_domain(), id).await?;
    Ok(StatusCode::NO_CONTENT)
}
