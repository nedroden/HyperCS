use chrono::Utc;
use hypercs_db::PgPool;
use hypercs_domain::{
    AuthUser as Principal,
    constants::{POST_BODY_MAX, POST_BODY_MIN, POSTS_PER_PAGE, REVISIONS_PER_PAGE},
    pagination::PageParams,
    policy,
};
use uuid::Uuid;

use crate::{
    dto::{PostDetail, PostRevisionPage, PostTopicContext, UpdatePostRequest},
    error::{ApiError, ApiResult},
    services::{flood, text, topic_service, view},
    state::FloodConfig,
};

pub async fn create_reply(
    db: &PgPool,
    flood_config: &FloodConfig,
    user: &Principal,
    topic_id: Uuid,
    req: &crate::dto::CreatePostRequest,
) -> ApiResult<Uuid> {
    let body = text::body("body", &req.body, POST_BODY_MIN, POST_BODY_MAX)?;

    let mut tx = db.begin().await?;
    hypercs_db::users::lock_posting(&mut tx, user.id).await?;
    let topic = hypercs_db::topics::lock_topic(&mut tx, topic_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if !policy::can_view_board(Some(user), topic.board_admin_only) {
        return Err(ApiError::not_found());
    }
    if !policy::can_reply_to_topic(Some(user), topic.board_admin_only, topic.locked) {
        return Err(ApiError::forbidden());
    }
    let now = Utc::now();
    flood::check_posting_allowed(&mut tx, flood_config, user.id, &body, now).await?;

    let post_id = Uuid::now_v7();
    hypercs_db::posts::insert_post(&mut *tx, post_id, topic_id, user.id, &body, now).await?;
    hypercs_db::topics::apply_reply_added(&mut *tx, topic_id, post_id, now).await?;
    hypercs_db::structure::bump_board_counts(&mut *tx, topic.board_id, 0, 1).await?;
    hypercs_db::structure::set_board_last_post(&mut *tx, topic.board_id, Some(post_id), Some(now))
        .await?;
    hypercs_db::users::bump_post_count(&mut *tx, user.id, 1).await?;
    tx.commit().await?;
    Ok(post_id)
}

enum NewBody<'a> {
    Text(&'a str),
    Revision(Uuid),
}

/// Locks the post row (`SELECT ... FOR UPDATE`), writes the pre-edit body as
/// a revision, then writes the new body, all in one transaction. Concurrent
/// edits therefore serialize and every version ends up in the history. The
/// revision id is generated after the lock is held, so revision ids (UUID v7)
/// are ordered like the edits themselves. A restore is just an edit whose new
/// body comes from an old revision.
async fn apply_edit_locked(
    db: &PgPool,
    user: &Principal,
    post_id: Uuid,
    source: NewBody<'_>,
) -> ApiResult<()> {
    let mut tx = db.begin().await?;
    let post = hypercs_db::posts::lock_post(&mut tx, post_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if !policy::can_view_board(Some(user), post.board_admin_only) {
        return Err(ApiError::not_found());
    }
    if !policy::can_edit_post(user, post.author_id, post.board_admin_only) {
        return Err(ApiError::forbidden());
    }
    let new_body = match source {
        NewBody::Text(body) => body.to_string(),
        NewBody::Revision(revision_id) => {
            let body = hypercs_db::posts::revision_body(&mut *tx, post_id, revision_id)
                .await?
                .ok_or_else(ApiError::not_found)?;
            // Limits may have changed since the revision was written.
            text::check_len("body", &body, POST_BODY_MIN, POST_BODY_MAX)?;
            body
        }
    };
    if new_body == post.body {
        return Ok(()); // nothing changed: no revision, no "edited" marker
    }
    hypercs_db::posts::apply_edit(
        &mut tx,
        post_id,
        Uuid::now_v7(),
        &post.body,
        &new_body,
        user.id,
        Utc::now(),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn update_post(
    db: &PgPool,
    user: &Principal,
    post_id: Uuid,
    req: &UpdatePostRequest,
) -> ApiResult<()> {
    let body = text::body("body", &req.body, POST_BODY_MIN, POST_BODY_MAX)?;
    apply_edit_locked(db, user, post_id, NewBody::Text(&body)).await
}

/// Writes an old revision's body back as a *new* edit (history is never
/// rewritten). Same authorization, validation and locking as any edit.
pub async fn restore_revision(
    db: &PgPool,
    user: &Principal,
    post_id: Uuid,
    revision_id: Uuid,
) -> ApiResult<()> {
    apply_edit_locked(db, user, post_id, NewBody::Revision(revision_id)).await
}

/// A single post with its topic context. Anyone who may view the board may
/// read it (guests included); otherwise it does not exist.
pub async fn get_post(
    db: &PgPool,
    viewer: Option<&Principal>,
    post_id: Uuid,
) -> ApiResult<PostDetail> {
    let row = hypercs_db::posts::get_one(db, post_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let topic = hypercs_db::topics::get_detail(db, row.topic_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if !policy::can_view_board(viewer, topic.board_admin_only) {
        return Err(ApiError::not_found());
    }
    let number = hypercs_db::posts::number_in_topic(db, row.topic_id, post_id).await?;
    let context = PostTopicContext {
        topic_id: topic.id,
        topic_title: topic.title,
        topic_slug: topic.slug,
        topic_locked: topic.locked,
        board_id: topic.board_id,
        board_name: topic.board_name,
        category_id: topic.category_id,
        category_name: topic.category_name,
        is_first_post: topic.first_post_id == post_id,
        page: ((number.max(1) - 1) / i64::from(POSTS_PER_PAGE) + 1) as u32,
    };
    Ok(PostDetail {
        post: view::post(row, number),
        context,
    })
}

pub async fn list_revisions(
    db: &PgPool,
    user: &Principal,
    post_id: Uuid,
    page: Option<u32>,
    per_page: Option<u32>,
) -> ApiResult<PostRevisionPage> {
    let post = hypercs_db::posts::get_for_read(db, post_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if !policy::can_view_board(Some(user), post.board_admin_only) {
        return Err(ApiError::not_found());
    }
    if !policy::can_view_post_history(user, post.author_id, post.board_admin_only) {
        return Err(ApiError::forbidden());
    }
    let params = PageParams::new(page, per_page, REVISIONS_PER_PAGE);
    let rows =
        hypercs_db::posts::list_revisions(db, post_id, params.limit(), params.offset()).await?;
    let total = hypercs_db::posts::count_revisions(db, post_id).await?;
    Ok(PostRevisionPage::new(
        rows.into_iter().map(view::revision).collect(),
        params,
        total,
    ))
}

/// Deletes a post. Deleting a topic's first post deletes the whole topic
/// (cascading its other posts); otherwise only the single post is removed.
/// Board/topic counters and authors' post counts are fixed in the same
/// transaction either way.
pub async fn delete_post(db: &PgPool, user: &Principal, post_id: Uuid) -> ApiResult<()> {
    let topic_id = hypercs_db::posts::topic_id_of(db, post_id)
        .await?
        .ok_or_else(ApiError::not_found)?;

    let mut tx = db.begin().await?;
    // Topic first, then the post (global lock order).
    let topic = hypercs_db::topics::lock_topic(&mut tx, topic_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let post = hypercs_db::posts::lock_post(&mut tx, post_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if !policy::can_view_board(Some(user), post.board_admin_only) {
        return Err(ApiError::not_found());
    }
    if !policy::can_delete_post(user, post.author_id, post.board_admin_only) {
        return Err(ApiError::forbidden());
    }

    if post.id == topic.first_post_id {
        topic_service::delete_topic_locked(&mut tx, topic.id, topic.board_id).await?;
    } else {
        hypercs_db::posts::delete(&mut *tx, post_id).await?;
        hypercs_db::topics::apply_post_removed(&mut *tx, topic.id).await?;
        hypercs_db::structure::bump_board_counts(&mut *tx, topic.board_id, 0, -1).await?;
        hypercs_db::structure::refresh_board_last_post(&mut *tx, topic.board_id).await?;
        hypercs_db::users::bump_post_count(&mut *tx, post.author_id, -1).await?;
    }
    tx.commit().await?;
    Ok(())
}
