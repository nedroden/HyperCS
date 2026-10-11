use chrono::{DateTime, Utc};
use hypercs_db::PgPool;
use hypercs_domain::{
    AuthUser as Principal,
    constants::{POST_BODY_MAX, POST_BODY_MIN, POSTS_PER_PAGE, TOPIC_TITLE_MAX, TOPIC_TITLE_MIN},
    pagination::PageParams,
    policy,
    validation::slugify,
};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{
    dto::{CreateTopicRequest, PostPage, TopicPageResponse, UpdateTopicRequest},
    error::{ApiError, ApiResult},
    services::{flood, text, view},
    state::FloodConfig,
};

const SLUG_MAX: usize = 80;

/// Inserts a topic with its first post and does all the counter bookkeeping
/// (board topic/post counts and last post, author post count). The caller
/// holds the transaction and the board row lock. Shared with the seed.
pub async fn insert_topic_with_first_post(
    conn: &mut PgConnection,
    board_id: Uuid,
    author_id: Uuid,
    title: &str,
    body: &str,
    now: DateTime<Utc>,
) -> sqlx::Result<(Uuid, Uuid)> {
    let topic_id = Uuid::now_v7();
    let post_id = Uuid::now_v7();
    let slug = slugify(title, SLUG_MAX, "topic");
    // topics.first_post_id/last_post_id reference posts through deferred
    // foreign keys, so the topic can be inserted first.
    hypercs_db::topics::insert_topic(
        &mut *conn, topic_id, board_id, title, &slug, author_id, post_id, now,
    )
    .await?;
    hypercs_db::posts::insert_post(&mut *conn, post_id, topic_id, author_id, body, now).await?;
    hypercs_db::structure::bump_board_counts(&mut *conn, board_id, 1, 1).await?;
    hypercs_db::structure::set_board_last_post(&mut *conn, board_id, Some(post_id), Some(now))
        .await?;
    hypercs_db::users::bump_post_count(&mut *conn, author_id, 1).await?;
    Ok((topic_id, post_id))
}

pub async fn create_topic(
    db: &PgPool,
    flood_config: &FloodConfig,
    user: &Principal,
    board_id: Uuid,
    req: &CreateTopicRequest,
) -> ApiResult<Uuid> {
    let title = text::line("title", &req.title, TOPIC_TITLE_MIN, TOPIC_TITLE_MAX)?;
    let body = text::body("body", &req.body, POST_BODY_MIN, POST_BODY_MAX)?;

    let mut tx = db.begin().await?;
    hypercs_db::users::lock_posting(&mut tx, user.id).await?;
    let board = hypercs_db::structure::lock_board_detail(&mut tx, board_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if !policy::can_view_board(Some(user), board.admin_only) {
        return Err(ApiError::not_found());
    }
    if !policy::can_create_topic(Some(user), board.admin_only) {
        return Err(ApiError::forbidden());
    }
    let now = Utc::now();
    flood::check_posting_allowed(&mut tx, flood_config, user.id, &body, now).await?;
    let (topic_id, _) =
        insert_topic_with_first_post(&mut tx, board_id, user.id, &title, &body, now).await?;
    tx.commit().await?;
    Ok(topic_id)
}

/// A topic page. Increments the view counter on every page request.
pub async fn get_topic(
    db: &PgPool,
    id: Uuid,
    viewer: Option<&Principal>,
    page: Option<u32>,
    per_page: Option<u32>,
) -> ApiResult<TopicPageResponse> {
    let topic = hypercs_db::topics::get_detail(db, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if !policy::can_view_board(viewer, topic.board_admin_only) {
        return Err(ApiError::not_found());
    }
    hypercs_db::topics::increment_view_count(db, id).await?;

    let params = PageParams::new(page, per_page, POSTS_PER_PAGE);
    let rows = hypercs_db::posts::list_for_topic(db, id, params.limit(), params.offset()).await?;
    let total = hypercs_db::posts::count_for_topic(db, id).await?;
    let first_number = params.offset() + 1;
    let items = rows
        .into_iter()
        .enumerate()
        .map(|(i, row)| view::post(row, first_number + i as i64))
        .collect();
    Ok(TopicPageResponse {
        topic: view::topic_detail(topic),
        posts: PostPage::new(items, params, total),
    })
}

/// Moderator/admin update: pin/unpin, lock/unlock and/or move to another
/// board, atomically (counters of both boards are fixed in the same
/// transaction).
pub async fn update_topic(
    db: &PgPool,
    user: &Principal,
    id: Uuid,
    req: &UpdateTopicRequest,
) -> ApiResult<()> {
    let mut tx = db.begin().await?;
    let topic = hypercs_db::topics::lock_topic(&mut tx, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if !policy::can_view_board(Some(user), topic.board_admin_only) {
        return Err(ApiError::not_found());
    }
    if !policy::can_moderate_topic(user, topic.board_admin_only) {
        return Err(ApiError::forbidden());
    }

    let new_board_id = req.board_id.filter(|b| *b != topic.board_id);
    if let Some(target_id) = new_board_id {
        // Lock both boards in ascending id order (deadlock avoidance).
        let (first, second) = if topic.board_id < target_id {
            (topic.board_id, target_id)
        } else {
            (target_id, topic.board_id)
        };
        let mut target_visible = false;
        for board_id in [first, second] {
            let board = hypercs_db::structure::lock_board_detail(&mut tx, board_id).await?;
            if board_id == target_id {
                target_visible =
                    board.is_some_and(|b| policy::can_view_board(Some(user), b.admin_only));
            }
        }
        if !target_visible {
            return Err(ApiError::validation(
                "boardId",
                "board_not_found",
                "The destination board does not exist.",
            ));
        }
        let post_count = topic.reply_count + 1;
        hypercs_db::topics::set_board(&mut *tx, id, target_id).await?;
        hypercs_db::structure::bump_board_counts(&mut *tx, topic.board_id, -1, -post_count).await?;
        hypercs_db::structure::bump_board_counts(&mut *tx, target_id, 1, post_count).await?;
        hypercs_db::structure::refresh_board_last_post(&mut *tx, topic.board_id).await?;
        hypercs_db::structure::refresh_board_last_post(&mut *tx, target_id).await?;
    }

    hypercs_db::topics::set_flags(
        &mut *tx,
        id,
        req.pinned.unwrap_or(topic.pinned),
        req.locked.unwrap_or(topic.locked),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Deletes a topic with all its posts and fixes every counter. The caller
/// holds the topic row lock.
pub async fn delete_topic_locked(
    conn: &mut PgConnection,
    topic_id: Uuid,
    board_id: Uuid,
) -> ApiResult<()> {
    let post_count = hypercs_db::posts::count_for_topic(&mut *conn, topic_id).await? as i32;
    let author_counts = hypercs_db::posts::author_counts_for_topic(&mut *conn, topic_id).await?;
    hypercs_db::topics::delete(&mut *conn, topic_id).await?;
    hypercs_db::structure::bump_board_counts(&mut *conn, board_id, -1, -post_count).await?;
    for (author_id, count) in author_counts {
        hypercs_db::users::bump_post_count(&mut *conn, author_id, -(count as i32)).await?;
    }
    hypercs_db::structure::refresh_board_last_post(&mut *conn, board_id).await?;
    Ok(())
}

/// Moderator/admin deletion of a whole topic (posts and revisions cascade).
pub async fn delete_topic(db: &PgPool, user: &Principal, id: Uuid) -> ApiResult<()> {
    let mut tx = db.begin().await?;
    let topic = hypercs_db::topics::lock_topic(&mut tx, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if !policy::can_view_board(Some(user), topic.board_admin_only) {
        return Err(ApiError::not_found());
    }
    if !policy::can_moderate_topic(user, topic.board_admin_only) {
        return Err(ApiError::forbidden());
    }
    delete_topic_locked(&mut tx, id, topic.board_id).await?;
    tx.commit().await?;
    Ok(())
}
