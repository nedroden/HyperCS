use chrono::Utc;
use hypercs_db::PgPool;
use hypercs_domain::validation::normalize_text;
use hypercs_domain::{AuthUser, DomainError, DomainResult};
use uuid::Uuid;

use crate::dto::{CreatePostRequest, UpdatePostRequest};

pub async fn create_reply(
    db: &PgPool,
    user: &AuthUser,
    topic_id: Uuid,
    req: &CreatePostRequest,
    account_created_at: chrono::DateTime<Utc>,
) -> DomainResult<Uuid> {
    let topic = hypercs_db::topics::get_moderation_info(db, topic_id)
        .await
        .map_err(|_| DomainError::Internal)?
        .ok_or(DomainError::NotFound)?;
    if !hypercs_domain::policy::can_reply_to_topic(Some(user), topic.admin_only, topic.locked) {
        return Err(DomainError::Forbidden);
    }
    let body = normalize_text(&req.body, true);
    super::moderation_guard::check_posting_allowed(db, user.id, account_created_at, &body).await?;

    let post_id = Uuid::now_v7();
    let now = Utc::now();
    let mut tx = db.begin().await.map_err(|_| DomainError::Internal)?;
    hypercs_db::posts::create_post(&mut *tx, post_id, topic_id, user.id, &body)
        .await
        .map_err(|_| DomainError::Internal)?;
    hypercs_db::topics::bump_reply_count(&mut *tx, topic_id, 1)
        .await
        .map_err(|_| DomainError::Internal)?;
    hypercs_db::topics::set_last_post(&mut *tx, topic_id, Some(post_id), Some(now))
        .await
        .map_err(|_| DomainError::Internal)?;
    hypercs_db::structure::bump_board_counts(&mut *tx, topic.board_id, 0, 1)
        .await
        .map_err(|_| DomainError::Internal)?;
    hypercs_db::structure::set_board_last_post(&mut *tx, topic.board_id, Some(post_id))
        .await
        .map_err(|_| DomainError::Internal)?;
    hypercs_db::users::bump_post_count(&mut *tx, user.id, 1)
        .await
        .map_err(|_| DomainError::Internal)?;
    tx.commit().await.map_err(|_| DomainError::Internal)?;

    Ok(post_id)
}

/// Locks the post row, writes the pre-edit body as a revision, then writes
/// the new body - all in one transaction so concurrent edits serialize and
/// neither version is lost.
async fn apply_edit_locked(
    db: &PgPool,
    user: &AuthUser,
    post_id: Uuid,
    new_body: String,
) -> DomainResult<()> {
    let mut tx = db.begin().await.map_err(|_| DomainError::Internal)?;
    let owner = hypercs_db::posts::get_owner(&mut *tx, post_id)
        .await
        .map_err(|_| DomainError::Internal)?
        .ok_or(DomainError::NotFound)?;
    if !hypercs_domain::policy::can_edit_post(user, owner.author_id) {
        return Err(DomainError::Forbidden);
    }
    let revision_id = Uuid::now_v7();
    hypercs_db::posts::apply_edit(
        &mut tx,
        post_id,
        revision_id,
        &owner.body,
        &new_body,
        user.id,
        Utc::now(),
    )
    .await
    .map_err(|_| DomainError::Internal)?;
    tx.commit().await.map_err(|_| DomainError::Internal)?;
    Ok(())
}

pub async fn update_post(
    db: &PgPool,
    user: &AuthUser,
    post_id: Uuid,
    req: &UpdatePostRequest,
) -> DomainResult<()> {
    let body = normalize_text(&req.body, true);
    apply_edit_locked(db, user, post_id, body).await
}

pub async fn restore_revision(
    db: &PgPool,
    user: &AuthUser,
    post_id: Uuid,
    revision_id: Uuid,
) -> DomainResult<()> {
    let revisions = hypercs_db::posts::list_revisions(db, post_id)
        .await
        .map_err(|_| DomainError::Internal)?;
    let revision = revisions
        .into_iter()
        .find(|r| r.id == revision_id)
        .ok_or(DomainError::NotFound)?;
    apply_edit_locked(db, user, post_id, revision.body).await
}

pub async fn list_revisions(
    db: &PgPool,
    user: &AuthUser,
    post_id: Uuid,
) -> DomainResult<Vec<hypercs_db::posts::PostRevisionRow>> {
    let post = hypercs_db::posts::get(db, post_id)
        .await
        .map_err(|_| DomainError::Internal)?
        .ok_or(DomainError::NotFound)?;
    if !hypercs_domain::policy::can_view_post_history(user, post.author_id) {
        return Err(DomainError::Forbidden);
    }
    hypercs_db::posts::list_revisions(db, post_id)
        .await
        .map_err(|_| DomainError::Internal)
}

/// Deletes a post. Deleting a topic's first post deletes the whole topic
/// (cascading its other posts); otherwise only the single post is removed.
/// Board/topic counters and the author's post count are fixed up in the
/// same transaction either way.
pub async fn delete_post(db: &PgPool, user: &AuthUser, post_id: Uuid) -> DomainResult<()> {
    let post = hypercs_db::posts::get(db, post_id)
        .await
        .map_err(|_| DomainError::Internal)?
        .ok_or(DomainError::NotFound)?;
    if !hypercs_domain::policy::can_delete_post(user, post.author_id) {
        return Err(DomainError::Forbidden);
    }
    let topic = hypercs_db::topics::get_moderation_info(db, post.topic_id)
        .await
        .map_err(|_| DomainError::Internal)?
        .ok_or(DomainError::Internal)?;
    let first_post_id = hypercs_db::posts::first_post_id_for_topic(db, post.topic_id)
        .await
        .map_err(|_| DomainError::Internal)?;

    let mut tx = db.begin().await.map_err(|_| DomainError::Internal)?;
    if first_post_id == Some(post_id) {
        let post_count = hypercs_db::posts::count_for_topic(&mut *tx, post.topic_id)
            .await
            .map_err(|_| DomainError::Internal)?;
        let author_counts = hypercs_db::posts::author_counts_for_topic(&mut *tx, post.topic_id)
            .await
            .map_err(|_| DomainError::Internal)?;
        hypercs_db::topics::delete(&mut *tx, post.topic_id)
            .await
            .map_err(|_| DomainError::Internal)?;
        hypercs_db::structure::bump_board_counts(
            &mut *tx,
            topic.board_id,
            -1,
            -(post_count as i32),
        )
        .await
        .map_err(|_| DomainError::Internal)?;
        for (author_id, count) in author_counts {
            hypercs_db::users::bump_post_count(&mut *tx, author_id, -(count as i32))
                .await
                .map_err(|_| DomainError::Internal)?;
        }
    } else {
        hypercs_db::posts::delete(&mut *tx, post_id)
            .await
            .map_err(|_| DomainError::Internal)?;
        hypercs_db::topics::bump_reply_count(&mut *tx, post.topic_id, -1)
            .await
            .map_err(|_| DomainError::Internal)?;
        hypercs_db::topics::recompute_topic_aggregates(&mut tx, post.topic_id)
            .await
            .map_err(|_| DomainError::Internal)?;
        hypercs_db::structure::bump_board_counts(&mut *tx, topic.board_id, 0, -1)
            .await
            .map_err(|_| DomainError::Internal)?;
        hypercs_db::users::bump_post_count(&mut *tx, post.author_id, -1)
            .await
            .map_err(|_| DomainError::Internal)?;
    }
    hypercs_db::structure::recompute_board_last_post(&mut *tx, topic.board_id)
        .await
        .map_err(|_| DomainError::Internal)?;
    tx.commit().await.map_err(|_| DomainError::Internal)?;
    Ok(())
}
