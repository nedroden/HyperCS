use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    dto::{NewsRequest, PageQuery},
    error::ApiResult,
    extract::ValidatedJson,
    services::news_service,
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/news", get(list_news).post(create_news))
        .route("/news/{slug}", get(get_news))
        .route(
            "/news/by-id/{id}",
            axum::routing::patch(update_news).delete(delete_news),
        )
}

#[utoipa::path(get, path = "/api/v1/news", responses((status = 200)), tag = "cms")]
async fn list_news(
    State(state): State<AppState>,
    Query(q): Query<PageQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let page = news_service::list(&state.db, q.page, q.per_page).await?;
    Ok(Json(serde_json::to_value(page).unwrap()))
}

#[utoipa::path(get, path = "/api/v1/news/{slug}", responses((status = 200)), tag = "cms")]
async fn get_news(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiResult<Json<hypercs_db::news::NewsItemRow>> {
    Ok(Json(news_service::get_by_slug(&state.db, &slug).await?))
}

#[utoipa::path(post, path = "/api/v1/news", request_body = NewsRequest, responses((status = 201)), tag = "admin")]
async fn create_news(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(req): ValidatedJson<NewsRequest>,
) -> ApiResult<impl IntoResponse> {
    let id = news_service::create(&state.db, &user.as_domain(), &req).await?;
    Ok((
        StatusCode::CREATED,
        [("Location", format!("/api/v1/news/by-id/{id}"))],
    ))
}

#[utoipa::path(patch, path = "/api/v1/news/by-id/{id}", request_body = NewsRequest, responses((status = 204)), tag = "admin")]
async fn update_news(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<NewsRequest>,
) -> ApiResult<StatusCode> {
    news_service::update(&state.db, &user.as_domain(), id, &req).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(delete, path = "/api/v1/news/by-id/{id}", responses((status = 204)), tag = "admin")]
async fn delete_news(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    news_service::delete(&state.db, &user.as_domain(), id).await?;
    Ok(StatusCode::NO_CONTENT)
}
