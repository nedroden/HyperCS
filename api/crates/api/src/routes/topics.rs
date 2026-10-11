use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::Response,
    routing::{get, post},
};
use uuid::Uuid;

use super::created;
use crate::{
    auth::{AuthUser, OptionalAuthUser},
    dto::{
        CreatePostRequest, CreateTopicRequest, CreatedResponse, PageQuery, PostDetail,
        PostRevisionPage, TopicPageResponse, UpdatePostRequest, UpdateTopicRequest,
    },
    error::{
        ApiResult, BadRequest, Conflict, Forbidden, NotFound, PayloadTooLarge, TooManyRequests,
        Unauthorized, Unprocessable, UnsupportedMediaType,
    },
    extract::{ApiPath, ApiQuery, ValidatedJson},
    services::{post_service, topic_service},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/boards/{id}/topics", post(create_topic))
        .route(
            "/topics/{id}",
            get(get_topic).patch(update_topic).delete(delete_topic),
        )
        .route("/topics/{id}/posts", post(create_reply))
        .route(
            "/posts/{id}",
            get(get_post).patch(update_post).delete(delete_post),
        )
        .route("/posts/{id}/revisions", get(list_revisions))
        .route(
            "/posts/{id}/revisions/{revisionId}/restore",
            post(restore_revision),
        )
}

/// Create a topic (title + first post) in a board.
#[utoipa::path(post, path = "/api/v1/boards/{id}/topics", operation_id = "create_topic", tag = "forum",
    security(("session_cookie" = [], "csrf_token" = [])),
    params(("id" = Uuid, Path, description = "Board id")),
    request_body = CreateTopicRequest,
    responses((status = 201, description = "Created.", body = CreatedResponse,
            headers(("Location" = String, description = "URL of the new topic"))),
        BadRequest, Unauthorized, Forbidden, NotFound, Conflict, PayloadTooLarge,
        UnsupportedMediaType, Unprocessable, TooManyRequests))]
async fn create_topic(
    State(state): State<AppState>,
    user: AuthUser,
    ApiPath(board_id): ApiPath<Uuid>,
    ValidatedJson(req): ValidatedJson<CreateTopicRequest>,
) -> ApiResult<Response> {
    let id = topic_service::create_topic(
        &state.db,
        &state.config.flood,
        &user.as_domain(),
        board_id,
        &req,
    )
    .await?;
    Ok(created(format!("/api/v1/topics/{id}"), id))
}

/// A topic with one page of posts. Every request counts as a view.
#[utoipa::path(get, path = "/api/v1/topics/{id}", operation_id = "get_topic", tag = "forum",
    security((), ("session_cookie" = [])),
    params(("id" = Uuid, Path, description = "Topic id"), PageQuery),
    responses((status = 200, body = TopicPageResponse), BadRequest, NotFound))]
async fn get_topic(
    State(state): State<AppState>,
    OptionalAuthUser(user): OptionalAuthUser,
    ApiPath(id): ApiPath<Uuid>,
    ApiQuery(q): ApiQuery<PageQuery>,
) -> ApiResult<Json<TopicPageResponse>> {
    let viewer = user.map(|u| u.as_domain());
    Ok(Json(
        topic_service::get_topic(&state.db, id, viewer.as_ref(), q.page, q.per_page).await?,
    ))
}

/// Moderator/admin: pin/unpin, lock/unlock and/or move the topic to another
/// board (`boardId`), all in one transaction.
#[utoipa::path(patch, path = "/api/v1/topics/{id}", operation_id = "update_topic", tag = "forum",
    security(("session_cookie" = [], "csrf_token" = [])),
    params(("id" = Uuid, Path, description = "Topic id")),
    request_body = UpdateTopicRequest,
    responses((status = 204, description = "Updated."),
        BadRequest, Unauthorized, Forbidden, NotFound, PayloadTooLarge, UnsupportedMediaType,
        Unprocessable))]
async fn update_topic(
    State(state): State<AppState>,
    user: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
    ValidatedJson(req): ValidatedJson<UpdateTopicRequest>,
) -> ApiResult<StatusCode> {
    topic_service::update_topic(&state.db, &user.as_domain(), id, &req).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Moderator/admin: hard-delete a topic with all its posts.
#[utoipa::path(delete, path = "/api/v1/topics/{id}", operation_id = "delete_topic", tag = "forum",
    security(("session_cookie" = [], "csrf_token" = [])),
    params(("id" = Uuid, Path, description = "Topic id")),
    responses((status = 204, description = "Deleted (cascade)."), Unauthorized, Forbidden, NotFound))]
async fn delete_topic(
    State(state): State<AppState>,
    user: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
) -> ApiResult<StatusCode> {
    topic_service::delete_topic(&state.db, &user.as_domain(), id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Reply to a topic. Locked topics accept replies from moderators and admins
/// only.
#[utoipa::path(post, path = "/api/v1/topics/{id}/posts", operation_id = "create_reply", tag = "forum",
    security(("session_cookie" = [], "csrf_token" = [])),
    params(("id" = Uuid, Path, description = "Topic id")),
    request_body = CreatePostRequest,
    responses((status = 201, description = "Created.", body = CreatedResponse,
            headers(("Location" = String, description = "URL of the new post"))),
        BadRequest, Unauthorized, Forbidden, NotFound, Conflict, PayloadTooLarge,
        UnsupportedMediaType, Unprocessable, TooManyRequests))]
async fn create_reply(
    State(state): State<AppState>,
    user: AuthUser,
    ApiPath(topic_id): ApiPath<Uuid>,
    ValidatedJson(req): ValidatedJson<CreatePostRequest>,
) -> ApiResult<Response> {
    let id = post_service::create_reply(
        &state.db,
        &state.config.flood,
        &user.as_domain(),
        topic_id,
        &req,
    )
    .await?;
    Ok(created(format!("/api/v1/posts/{id}"), id))
}

/// A single post with its topic, board and category context.
#[utoipa::path(get, path = "/api/v1/posts/{id}", operation_id = "get_post", tag = "forum",
    security((), ("session_cookie" = [])),
    params(("id" = Uuid, Path, description = "Post id")),
    responses((status = 200, body = PostDetail), BadRequest, NotFound))]
async fn get_post(
    State(state): State<AppState>,
    OptionalAuthUser(user): OptionalAuthUser,
    ApiPath(id): ApiPath<Uuid>,
) -> ApiResult<Json<PostDetail>> {
    let viewer = user.map(|u| u.as_domain());
    Ok(Json(
        post_service::get_post(&state.db, viewer.as_ref(), id).await?,
    ))
}

/// Edit a post (author, moderator or admin). The previous body is kept as a
/// revision; an unchanged body is a no-op.
#[utoipa::path(patch, path = "/api/v1/posts/{id}", operation_id = "update_post", tag = "forum",
    security(("session_cookie" = [], "csrf_token" = [])),
    params(("id" = Uuid, Path, description = "Post id")),
    request_body = UpdatePostRequest,
    responses((status = 204, description = "Edited."),
        BadRequest, Unauthorized, Forbidden, NotFound, PayloadTooLarge, UnsupportedMediaType,
        Unprocessable))]
async fn update_post(
    State(state): State<AppState>,
    user: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
    ValidatedJson(req): ValidatedJson<UpdatePostRequest>,
) -> ApiResult<StatusCode> {
    post_service::update_post(&state.db, &user.as_domain(), id, &req).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Delete a post (author, moderator or admin). Deleting a topic's first post
/// deletes the whole topic.
#[utoipa::path(delete, path = "/api/v1/posts/{id}", operation_id = "delete_post", tag = "forum",
    security(("session_cookie" = [], "csrf_token" = [])),
    params(("id" = Uuid, Path, description = "Post id")),
    responses((status = 204, description = "Deleted."), Unauthorized, Forbidden, NotFound))]
async fn delete_post(
    State(state): State<AppState>,
    user: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
) -> ApiResult<StatusCode> {
    post_service::delete_post(&state.db, &user.as_domain(), id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// A post's edit history, newest first (author, moderator or admin).
#[utoipa::path(get, path = "/api/v1/posts/{id}/revisions", operation_id = "list_revisions", tag = "forum",
    security(("session_cookie" = [])),
    params(("id" = Uuid, Path, description = "Post id"), PageQuery),
    responses((status = 200, body = PostRevisionPage), BadRequest, Unauthorized, Forbidden, NotFound))]
async fn list_revisions(
    State(state): State<AppState>,
    user: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
    ApiQuery(q): ApiQuery<PageQuery>,
) -> ApiResult<Json<PostRevisionPage>> {
    Ok(Json(
        post_service::list_revisions(&state.db, &user.as_domain(), id, q.page, q.per_page).await?,
    ))
}

/// Restore an old version: a normal edit that writes the revision's body back
/// and records the current body as a new revision (history is never
/// rewritten).
#[utoipa::path(post, path = "/api/v1/posts/{id}/revisions/{revisionId}/restore",
    operation_id = "restore_revision", tag = "forum",
    security(("session_cookie" = [], "csrf_token" = [])),
    params(("id" = Uuid, Path, description = "Post id"),
           ("revisionId" = Uuid, Path, description = "Revision id")),
    responses((status = 204, description = "Restored as a new edit."),
        Unauthorized, Forbidden, NotFound, Unprocessable))]
async fn restore_revision(
    State(state): State<AppState>,
    user: AuthUser,
    ApiPath((id, revision_id)): ApiPath<(Uuid, Uuid)>,
) -> ApiResult<StatusCode> {
    post_service::restore_revision(&state.db, &user.as_domain(), id, revision_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
