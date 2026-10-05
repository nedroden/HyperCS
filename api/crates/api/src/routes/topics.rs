use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use hypercs_domain::DomainError;
use uuid::Uuid;

use crate::{
    auth::{AuthUser, OptionalAuthUser},
    dto::{
        CreatePostRequest, CreateTopicRequest, ModerateTopicRequest, MoveTopicRequest, PageQuery,
        UpdatePostRequest,
    },
    error::ApiResult,
    extract::ValidatedJson,
    services::{post_service, topic_service},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/boards/{id}/topics", post(create_topic))
        .route(
            "/topics/{id}",
            get(get_topic).patch(moderate_topic).delete(delete_topic),
        )
        .route("/topics/{id}/move", post(move_topic))
        .route("/topics/{id}/posts", post(create_reply))
        .route(
            "/posts/{id}",
            axum::routing::patch(update_post).delete(delete_post),
        )
        .route("/posts/{id}/revisions", get(list_revisions))
        .route(
            "/posts/{id}/revisions/{revisionId}/restore",
            post(restore_revision),
        )
}

async fn account_created_at(
    state: &AppState,
    user_id: Uuid,
) -> ApiResult<chrono::DateTime<chrono::Utc>> {
    hypercs_db::users::created_at_by_id(&state.db, user_id)
        .await
        .map_err(crate::error::ApiError::Sqlx)?
        .ok_or(crate::error::ApiError::Domain(DomainError::Internal))
}

#[utoipa::path(post, path = "/api/v1/boards/{id}/topics", request_body = CreateTopicRequest, responses((status = 201)), tag = "forum")]
async fn create_topic(
    State(state): State<AppState>,
    user: AuthUser,
    Path(board_id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<CreateTopicRequest>,
) -> ApiResult<impl IntoResponse> {
    let created_at = account_created_at(&state, user.id).await?;
    let id = topic_service::create_topic(&state.db, &user.as_domain(), board_id, &req, created_at)
        .await?;
    Ok((
        StatusCode::CREATED,
        [("Location", format!("/api/v1/topics/{id}"))],
    ))
}

#[utoipa::path(get, path = "/api/v1/topics/{id}", responses((status = 200)), tag = "forum")]
async fn get_topic(
    State(state): State<AppState>,
    OptionalAuthUser(user): OptionalAuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<PageQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let domain_user = user.map(|u| u.as_domain());
    let (topic, posts) =
        topic_service::get_topic(&state.db, id, domain_user.as_ref(), q.page, q.per_page).await?;
    Ok(Json(serde_json::json!({ "topic": topic, "posts": posts })))
}

#[utoipa::path(patch, path = "/api/v1/topics/{id}", request_body = ModerateTopicRequest, responses((status = 204)), tag = "forum")]
async fn moderate_topic(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<ModerateTopicRequest>,
) -> ApiResult<StatusCode> {
    topic_service::moderate(&state.db, &user.as_domain(), id, req.pinned, req.locked).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post, path = "/api/v1/topics/{id}/move", request_body = MoveTopicRequest, responses((status = 204)), tag = "forum")]
async fn move_topic(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<MoveTopicRequest>,
) -> ApiResult<StatusCode> {
    topic_service::move_topic(&state.db, &user.as_domain(), id, req.board_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(delete, path = "/api/v1/topics/{id}", responses((status = 204)), tag = "forum")]
async fn delete_topic(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    topic_service::delete_topic(&state.db, &user.as_domain(), id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post, path = "/api/v1/topics/{id}/posts", request_body = CreatePostRequest, responses((status = 201)), tag = "forum")]
async fn create_reply(
    State(state): State<AppState>,
    user: AuthUser,
    Path(topic_id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<CreatePostRequest>,
) -> ApiResult<impl IntoResponse> {
    let created_at = account_created_at(&state, user.id).await?;
    let id = post_service::create_reply(&state.db, &user.as_domain(), topic_id, &req, created_at)
        .await?;
    Ok((
        StatusCode::CREATED,
        [("Location", format!("/api/v1/posts/{id}"))],
    ))
}

#[utoipa::path(patch, path = "/api/v1/posts/{id}", request_body = UpdatePostRequest, responses((status = 204)), tag = "forum")]
async fn update_post(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<UpdatePostRequest>,
) -> ApiResult<StatusCode> {
    post_service::update_post(&state.db, &user.as_domain(), id, &req).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(delete, path = "/api/v1/posts/{id}", responses((status = 204)), tag = "forum")]
async fn delete_post(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    post_service::delete_post(&state.db, &user.as_domain(), id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/api/v1/posts/{id}/revisions", responses((status = 200)), tag = "forum")]
async fn list_revisions(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<hypercs_db::posts::PostRevisionRow>>> {
    let revisions = post_service::list_revisions(&state.db, &user.as_domain(), id).await?;
    Ok(Json(revisions))
}

#[utoipa::path(post, path = "/api/v1/posts/{id}/revisions/{revisionId}/restore", responses((status = 204)), tag = "forum")]
async fn restore_revision(
    State(state): State<AppState>,
    user: AuthUser,
    Path((id, revision_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<StatusCode> {
    post_service::restore_revision(&state.db, &user.as_domain(), id, revision_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
