use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgConnection, PgExecutor};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct TopicDetailRow {
    pub id: Uuid,
    pub board_id: Uuid,
    pub board_name: String,
    pub category_id: Uuid,
    pub category_name: String,
    /// Effective flag: the board's own OR its category's.
    pub board_admin_only: bool,
    pub title: String,
    pub slug: String,
    pub first_post_id: Uuid,
    pub author_id: Uuid,
    pub author_username: String,
    pub author_display_name: String,
    pub author_role: String,
    pub pinned: bool,
    pub locked: bool,
    pub reply_count: i32,
    pub view_count: i32,
    pub created_at: DateTime<Utc>,
}

/// A topic row locked `FOR UPDATE` together with what authorization needs.
#[derive(Debug, Clone, FromRow)]
pub struct TopicLockRow {
    pub id: Uuid,
    pub board_id: Uuid,
    pub locked: bool,
    pub pinned: bool,
    pub first_post_id: Uuid,
    pub reply_count: i32,
    pub board_admin_only: bool,
}

#[derive(Debug, Clone, FromRow)]
pub struct TopicListRow {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub pinned: bool,
    pub locked: bool,
    pub reply_count: i32,
    pub view_count: i32,
    pub created_at: DateTime<Utc>,
    pub author_id: Uuid,
    pub author_username: String,
    pub author_display_name: String,
    pub author_role: String,
    pub lp_post_id: Option<Uuid>,
    pub lp_created_at: Option<DateTime<Utc>>,
    pub lp_author_id: Option<Uuid>,
    pub lp_author_username: Option<String>,
    pub lp_author_display_name: Option<String>,
    pub lp_author_role: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_topic<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    board_id: Uuid,
    title: &str,
    slug: &str,
    author_id: Uuid,
    first_post_id: Uuid,
    created_at: DateTime<Utc>,
) -> sqlx::Result<()> {
    // The first post is also the last post until a reply arrives. The FKs
    // to posts are deferred, so the post may be inserted after the topic.
    sqlx::query(
        "insert into topics (id, board_id, title, slug, author_id, first_post_id, \
            last_post_id, last_post_at, created_at) \
         values ($1, $2, $3, $4, $5, $6, $6, $7, $7)",
    )
    .bind(id)
    .bind(board_id)
    .bind(title)
    .bind(slug)
    .bind(author_id)
    .bind(first_post_id)
    .bind(created_at)
    .execute(ex)
    .await?;
    Ok(())
}

pub async fn get_detail<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
) -> sqlx::Result<Option<TopicDetailRow>> {
    sqlx::query_as(
        "select t.id, t.board_id, b.name as board_name, c.id as category_id, \
            c.name as category_name, (b.admin_only or c.admin_only) as board_admin_only, \
            t.title, t.slug, t.first_post_id, t.author_id, u.username as author_username, \
            u.display_name as author_display_name, u.role as author_role, \
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

/// Locks the topic row. Lock order across the codebase: posting advisory
/// lock, then topic, then board(s), then user counters.
pub async fn lock_topic(ex: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<TopicLockRow>> {
    sqlx::query_as(
        "select t.id, t.board_id, t.locked, t.pinned, t.first_post_id, t.reply_count, \
            (b.admin_only or c.admin_only) as board_admin_only \
         from topics t \
         join boards b on b.id = t.board_id \
         join categories c on c.id = b.category_id \
         where t.id = $1 for update of t",
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
) -> sqlx::Result<Vec<TopicListRow>> {
    sqlx::query_as(
        "select t.id, t.title, t.slug, t.pinned, t.locked, t.reply_count, t.view_count, \
            t.created_at, u.id as author_id, u.username as author_username, \
            u.display_name as author_display_name, u.role as author_role, \
            p.id as lp_post_id, p.created_at as lp_created_at, \
            lu.id as lp_author_id, lu.username as lp_author_username, \
            lu.display_name as lp_author_display_name, lu.role as lp_author_role \
         from topics t \
         join users u on u.id = t.author_id \
         left join posts p on p.id = t.last_post_id \
         left join users lu on lu.id = p.author_id \
         where t.board_id = $1 \
         order by t.pinned desc, t.last_post_at desc, t.id desc \
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

pub async fn set_flags<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    pinned: bool,
    locked: bool,
) -> sqlx::Result<()> {
    sqlx::query("update topics set pinned = $2, locked = $3 where id = $1")
        .bind(id)
        .bind(pinned)
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

/// A reply was added: bump the reply count and move the last-post pointer.
pub async fn apply_reply_added<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    post_id: Uuid,
    post_created_at: DateTime<Utc>,
) -> sqlx::Result<()> {
    sqlx::query(
        "update topics set reply_count = reply_count + 1, last_post_id = $2, last_post_at = $3 \
         where id = $1",
    )
    .bind(id)
    .bind(post_id)
    .bind(post_created_at)
    .execute(ex)
    .await?;
    Ok(())
}

/// A (non-first) post was removed: drop the reply count by one and recompute
/// the last-post pointer from the remaining posts.
pub async fn apply_post_removed<'e>(ex: impl PgExecutor<'e>, id: Uuid) -> sqlx::Result<()> {
    sqlx::query(
        "update topics set reply_count = greatest(reply_count - 1, 0), \
            last_post_id = (select p.id from posts p where p.topic_id = $1 order by p.id desc limit 1), \
            last_post_at = coalesce( \
                (select p.created_at from posts p where p.topic_id = $1 order by p.id desc limit 1), \
                last_post_at) \
         where id = $1",
    )
    .bind(id)
    .execute(ex)
    .await?;
    Ok(())
}

pub async fn count_all<'e>(ex: impl PgExecutor<'e>) -> sqlx::Result<i64> {
    let row: (i64,) = sqlx::query_as("select count(*) from topics")
        .fetch_one(ex)
        .await?;
    Ok(row.0)
}
