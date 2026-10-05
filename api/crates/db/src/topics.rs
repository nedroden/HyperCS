use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, PgExecutor};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TopicDetailRow {
    pub id: Uuid,
    pub board_id: Uuid,
    pub board_name: String,
    pub category_id: Uuid,
    pub category_name: String,
    pub title: String,
    pub slug: String,
    pub author_id: Uuid,
    pub author_username: String,
    pub author_display_name: String,
    pub pinned: bool,
    pub locked: bool,
    pub reply_count: i32,
    pub view_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct TopicModerationRow {
    pub board_id: Uuid,
    pub author_id: Uuid,
    pub locked: bool,
    pub admin_only: bool,
}

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TopicListItemRow {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub author_id: Uuid,
    pub author_username: String,
    pub author_display_name: String,
    pub pinned: bool,
    pub locked: bool,
    pub reply_count: i32,
    pub view_count: i32,
    pub created_at: DateTime<Utc>,
    pub last_post_id: Option<Uuid>,
    pub last_post_at: Option<DateTime<Utc>>,
    pub last_post_author_id: Option<Uuid>,
    pub last_post_author_username: Option<String>,
    pub last_post_author_display_name: Option<String>,
}

pub async fn create_topic<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    board_id: Uuid,
    title: &str,
    slug: &str,
    author_id: Uuid,
) -> sqlx::Result<()> {
    sqlx::query(
        "insert into topics (id, board_id, title, slug, author_id) values ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(board_id)
    .bind(title)
    .bind(slug)
    .bind(author_id)
    .execute(ex)
    .await?;
    Ok(())
}

pub async fn get_detail<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
) -> sqlx::Result<Option<TopicDetailRow>> {
    sqlx::query_as::<_, TopicDetailRow>(
        "select t.id, t.board_id, b.name as board_name, c.id as category_id, c.name as category_name, \
         t.title, t.slug, t.author_id, u.username as author_username, u.display_name as author_display_name, \
         t.pinned, t.locked, t.reply_count, t.view_count, t.created_at \
         from topics t \
         join boards b on b.id = t.board_id \
         join categories c on c.id = b.category_id \
         join users u on u.id = t.author_id \
         where t.id = $1",
    )
    .bind(id)
    .fetch_optional(ex)
    .await
}

pub async fn get_moderation_info<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
) -> sqlx::Result<Option<TopicModerationRow>> {
    sqlx::query_as::<_, TopicModerationRow>(
        "select t.board_id, t.author_id, t.locked, b.admin_only \
         from topics t join boards b on b.id = t.board_id where t.id = $1",
    )
    .bind(id)
    .fetch_optional(ex)
    .await
}

pub async fn list_for_board<'e>(
    ex: impl PgExecutor<'e>,
    board_id: Uuid,
    limit: i64,
    offset: i64,
) -> sqlx::Result<Vec<TopicListItemRow>> {
    sqlx::query_as::<_, TopicListItemRow>(
        "select t.id, t.title, t.slug, t.author_id, u.username as author_username, \
         u.display_name as author_display_name, t.pinned, t.locked, t.reply_count, t.view_count, t.created_at, \
         p.id as last_post_id, p.created_at as last_post_at, \
         lu.id as last_post_author_id, lu.username as last_post_author_username, \
         lu.display_name as last_post_author_display_name \
         from topics t \
         join users u on u.id = t.author_id \
         left join posts p on p.id = t.last_post_id \
         left join users lu on lu.id = p.author_id \
         where t.board_id = $1 \
         order by t.pinned desc, coalesce(p.created_at, t.created_at) desc \
         limit $2 offset $3",
    )
    .bind(board_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(ex)
    .await
}

pub async fn count_for_board<'e>(ex: impl PgExecutor<'e>, board_id: Uuid) -> sqlx::Result<i64> {
    let row: (i64,) = sqlx::query_as("select count(*) from topics where board_id = $1")
        .bind(board_id)
        .fetch_one(ex)
        .await?;
    Ok(row.0)
}

pub async fn increment_view_count<'e>(ex: impl PgExecutor<'e>, id: Uuid) -> sqlx::Result<()> {
    sqlx::query("update topics set view_count = view_count + 1 where id = $1")
        .bind(id)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn set_pinned<'e>(ex: impl PgExecutor<'e>, id: Uuid, pinned: bool) -> sqlx::Result<()> {
    sqlx::query("update topics set pinned = $2 where id = $1")
        .bind(id)
        .bind(pinned)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn set_locked<'e>(ex: impl PgExecutor<'e>, id: Uuid, locked: bool) -> sqlx::Result<()> {
    sqlx::query("update topics set locked = $2 where id = $1")
        .bind(id)
        .bind(locked)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn set_board<'e>(ex: impl PgExecutor<'e>, id: Uuid, board_id: Uuid) -> sqlx::Result<()> {
    sqlx::query("update topics set board_id = $2 where id = $1")
        .bind(id)
        .bind(board_id)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn delete<'e>(ex: impl PgExecutor<'e>, id: Uuid) -> sqlx::Result<u64> {
    let result = sqlx::query("delete from topics where id = $1")
        .bind(id)
        .execute(ex)
        .await?;
    Ok(result.rows_affected())
}

pub async fn bump_reply_count<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    delta: i32,
) -> sqlx::Result<()> {
    sqlx::query("update topics set reply_count = reply_count + $2 where id = $1")
        .bind(id)
        .bind(delta)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn set_last_post<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    last_post_id: Option<Uuid>,
    last_post_at: Option<DateTime<Utc>>,
) -> sqlx::Result<()> {
    sqlx::query("update topics set last_post_id = $2, last_post_at = $3 where id = $1")
        .bind(id)
        .bind(last_post_id)
        .bind(last_post_at)
        .execute(ex)
        .await?;
    Ok(())
}

/// Recomputes a topic's `last_post_id`/`last_post_at` and `reply_count` from
/// its remaining posts (used by delete and by the `recount` CLI).
pub async fn recompute_topic_aggregates(ex: &mut sqlx::PgConnection, id: Uuid) -> sqlx::Result<()> {
    sqlx::query(
        "update topics set reply_count = greatest(( \
            select count(*) from posts where topic_id = $1 \
         ) - 1, 0) where id = $1",
    )
    .bind(id)
    .execute(&mut *ex)
    .await?;
    sqlx::query(
        "update topics t set last_post_id = lp.id, last_post_at = lp.created_at \
         from (select id, created_at from posts where topic_id = $1 order by created_at desc limit 1) lp \
         where t.id = $1",
    )
    .bind(id)
    .execute(&mut *ex)
    .await?;
    Ok(())
}

pub async fn count_all<'e>(ex: impl PgExecutor<'e>) -> sqlx::Result<i64> {
    let row: (i64,) = sqlx::query_as("select count(*) from topics")
        .fetch_one(ex)
        .await?;
    Ok(row.0)
}

pub async fn all_ids<'e>(ex: impl PgExecutor<'e>) -> sqlx::Result<Vec<Uuid>> {
    let rows: Vec<(Uuid,)> = sqlx::query_as("select id from topics")
        .fetch_all(ex)
        .await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}
