use chrono::Utc;
use hypercs_db::PgPool;
use hypercs_db::pagination::{Page, PageParams};
use hypercs_domain::constants::POSTS_PER_PAGE;
use hypercs_domain::validation::{normalize_text, slugify};
use hypercs_domain::{AuthUser, DomainError, DomainResult};
use uuid::Uuid;

use crate::dto::CreateTopicRequest;

pub async fn create_topic(
    db: &PgPool,
    user: &AuthUser,
    board_id: Uuid,
    req: &CreateTopicRequest,
    account_created_at: chrono::DateTime<Utc>,
) -> DomainResult<Uuid> {
    let board = hypercs_db::structure::get_board_detail(db, board_id)
        .await
        .map_err(|_| DomainError::Internal)?
        .ok_or(DomainError::NotFound)?;
    if !hypercs_domain::policy::can_create_topic(Some(user), board.admin_only) {
        return Err(DomainError::Forbidden);
    }
    let title = normalize_text(&req.title, false);
    let body = normalize_text(&req.body, true);
    super::moderation_guard::check_posting_allowed(db, user.id, account_created_at, &body).await?;

    let slug = slugify(&title, 80);
    let topic_id = Uuid::now_v7();
    let post_id = Uuid::now_v7();
    let now = Utc::now();

    let mut tx = db.begin().await.map_err(|_| DomainError::Internal)?;
    hypercs_db::topics::create_topic(&mut *tx, topic_id, board_id, &title, &slug, user.id)
        .await
        .map_err(|_| DomainError::Internal)?;
    hypercs_db::posts::create_post(&mut *tx, post_id, topic_id, user.id, &body)
        .await
        .map_err(|_| DomainError::Internal)?;
    hypercs_db::topics::set_last_post(&mut *tx, topic_id, Some(post_id), Some(now))
        .await
        .map_err(|_| DomainError::Internal)?;
    hypercs_db::structure::bump_board_counts(&mut *tx, board_id, 1, 1)
        .await
        .map_err(|_| DomainError::Internal)?;
    hypercs_db::structure::set_board_last_post(&mut *tx, board_id, Some(post_id))
        .await
        .map_err(|_| DomainError::Internal)?;
    hypercs_db::users::bump_post_count(&mut *tx, user.id, 1)
        .await
        .map_err(|_| DomainError::Internal)?;
    tx.commit().await.map_err(|_| DomainError::Internal)?;

    Ok(topic_id)
}

pub async fn get_topic(
    db: &PgPool,
    id: Uuid,
    viewer: Option<&AuthUser>,
    page: Option<u32>,
    per_page: Option<u32>,
) -> DomainResult<(
    hypercs_db::topics::TopicDetailRow,
    Page<hypercs_db::posts::PostRow>,
)> {
    let topic = hypercs_db::topics::get_detail(db, id)
        .await
        .map_err(|_| DomainError::Internal)?
        .ok_or(DomainError::NotFound)?;
    let board = hypercs_db::structure::get_board_detail(db, topic.board_id)
        .await
        .map_err(|_| DomainError::Internal)?
        .ok_or(DomainError::Internal)?;
    if !hypercs_domain::policy::can_view_board(viewer, board.admin_only) {
        return Err(DomainError::NotFound);
    }
    hypercs_db::topics::increment_view_count(db, id)
        .await
        .map_err(|_| DomainError::Internal)?;

    let params = PageParams::new(page, per_page, POSTS_PER_PAGE);
    let posts = hypercs_db::posts::list_for_topic(db, id, params.limit(), params.offset())
        .await
        .map_err(|_| DomainError::Internal)?;
    let total = hypercs_db::posts::count_for_topic(db, id)
        .await
        .map_err(|_| DomainError::Internal)?;
    Ok((topic, Page::new(posts, params, total)))
}

pub async fn moderate(
    db: &PgPool,
    user: &AuthUser,
    id: Uuid,
    pinned: Option<bool>,
    locked: Option<bool>,
) -> DomainResult<()> {
    if !hypercs_domain::policy::can_moderate_topic(user) {
        return Err(DomainError::Forbidden);
    }
    hypercs_db::topics::get_moderation_info(db, id)
        .await
        .map_err(|_| DomainError::Internal)?
        .ok_or(DomainError::NotFound)?;
    if let Some(pinned) = pinned {
        hypercs_db::topics::set_pinned(db, id, pinned)
            .await
            .map_err(|_| DomainError::Internal)?;
    }
    if let Some(locked) = locked {
        hypercs_db::topics::set_locked(db, id, locked)
            .await
            .map_err(|_| DomainError::Internal)?;
    }
    Ok(())
}

pub async fn move_topic(
    db: &PgPool,
    user: &AuthUser,
    id: Uuid,
    new_board_id: Uuid,
) -> DomainResult<()> {
    if !hypercs_domain::policy::can_moderate_topic(user) {
        return Err(DomainError::Forbidden);
    }
    let info = hypercs_db::topics::get_moderation_info(db, id)
        .await
        .map_err(|_| DomainError::Internal)?
        .ok_or(DomainError::NotFound)?;
    if info.board_id == new_board_id {
        return Ok(());
    }
    hypercs_db::structure::get_board_detail(db, new_board_id)
        .await
        .map_err(|_| DomainError::Internal)?
        .ok_or(DomainError::NotFound)?;

    let post_count = hypercs_db::posts::count_for_topic(db, id)
        .await
        .map_err(|_| DomainError::Internal)?;

    let mut tx = db.begin().await.map_err(|_| DomainError::Internal)?;
    hypercs_db::topics::set_board(&mut *tx, id, new_board_id)
        .await
        .map_err(|_| DomainError::Internal)?;
    hypercs_db::structure::bump_board_counts(&mut *tx, info.board_id, -1, -(post_count as i32))
        .await
        .map_err(|_| DomainError::Internal)?;
    hypercs_db::structure::bump_board_counts(&mut *tx, new_board_id, 1, post_count as i32)
        .await
        .map_err(|_| DomainError::Internal)?;
    hypercs_db::structure::recompute_board_last_post(&mut *tx, info.board_id)
        .await
        .map_err(|_| DomainError::Internal)?;
    hypercs_db::structure::recompute_board_last_post(&mut *tx, new_board_id)
        .await
        .map_err(|_| DomainError::Internal)?;
    tx.commit().await.map_err(|_| DomainError::Internal)?;
    Ok(())
}

/// Direct moderator/admin deletion of an entire topic (all its posts cascade
/// via the foreign key); counters on its board are fixed up in the same
/// transaction.
pub async fn delete_topic(db: &PgPool, user: &AuthUser, id: Uuid) -> DomainResult<()> {
    if !hypercs_domain::policy::can_moderate_topic(user) {
        return Err(DomainError::Forbidden);
    }
    let info = hypercs_db::topics::get_moderation_info(db, id)
        .await
        .map_err(|_| DomainError::Internal)?
        .ok_or(DomainError::NotFound)?;
    let post_count = hypercs_db::posts::count_for_topic(db, id)
        .await
        .map_err(|_| DomainError::Internal)?;
    let author_counts = hypercs_db::posts::author_counts_for_topic(db, id)
        .await
        .map_err(|_| DomainError::Internal)?;

    let mut tx = db.begin().await.map_err(|_| DomainError::Internal)?;
    hypercs_db::topics::delete(&mut *tx, id)
        .await
        .map_err(|_| DomainError::Internal)?;
    hypercs_db::structure::bump_board_counts(&mut *tx, info.board_id, -1, -(post_count as i32))
        .await
        .map_err(|_| DomainError::Internal)?;
    for (author_id, count) in author_counts {
        hypercs_db::users::bump_post_count(&mut *tx, author_id, -(count as i32))
            .await
            .map_err(|_| DomainError::Internal)?;
    }
    hypercs_db::structure::recompute_board_last_post(&mut *tx, info.board_id)
        .await
        .map_err(|_| DomainError::Internal)?;
    tx.commit().await.map_err(|_| DomainError::Internal)?;
    Ok(())
}
