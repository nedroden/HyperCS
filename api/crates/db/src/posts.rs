use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgConnection, PgExecutor};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct PostRow {
    pub id: Uuid,
    pub topic_id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub author_id: Uuid,
    pub author_username: String,
    pub author_display_name: String,
    pub author_role: String,
    pub author_post_count: i32,
    pub author_joined_at: DateTime<Utc>,
    pub editor_id: Option<Uuid>,
    pub editor_username: Option<String>,
    pub editor_display_name: Option<String>,
    pub editor_role: Option<String>,
}

/// A post locked `FOR UPDATE` with its board's effective admin-only flag.
#[derive(Debug, Clone, FromRow)]
pub struct PostLockRow {
    pub id: Uuid,
    pub topic_id: Uuid,
    pub author_id: Uuid,
    pub body: String,
    pub board_admin_only: bool,
}

#[derive(Debug, Clone, FromRow)]
pub struct PostRevisionRow {
    pub id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub editor_id: Option<Uuid>,
    pub editor_username: Option<String>,
    pub editor_display_name: Option<String>,
    pub editor_role: Option<String>,
}

pub async fn insert_post<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    topic_id: Uuid,
    author_id: Uuid,
    body: &str,
    created_at: DateTime<Utc>,
) -> sqlx::Result<()> {
    sqlx::query(
        "insert into posts (id, topic_id, author_id, body, created_at) values ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(topic_id)
    .bind(author_id)
    .bind(body)
    .bind(created_at)
    .execute(ex)
    .await?;
    Ok(())
}

/// Id of the topic a post belongs to (to take the topic lock first).
pub async fn topic_id_of<'e>(ex: impl PgExecutor<'e>, id: Uuid) -> sqlx::Result<Option<Uuid>> {
    let row: Option<(Uuid,)> = sqlx::query_as("select topic_id from posts where id = $1")
        .bind(id)
        .fetch_optional(ex)
        .await?;
    Ok(row.map(|r| r.0))
}

const POST_LOCK_SELECT: &str = "select p.id, p.topic_id, p.author_id, p.body, \
    (b.admin_only or c.admin_only) as board_admin_only \
    from posts p join topics t on t.id = p.topic_id \
    join boards b on b.id = t.board_id join categories c on c.id = b.category_id";

/// Locks the post row (`FOR UPDATE OF p`).
pub async fn lock_post(ex: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<PostLockRow>> {
    sqlx::query_as(const_format::concatcp!(
        POST_LOCK_SELECT,
        " where p.id = $1 for update of p"
    ))
    .bind(id)
    .fetch_optional(ex)
    .await
}

/// Read-only variant (no lock), for authorization of history reads.
pub async fn get_for_read<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
) -> sqlx::Result<Option<PostLockRow>> {
    sqlx::query_as(const_format::concatcp!(
        POST_LOCK_SELECT,
        " where p.id = $1"
    ))
    .bind(id)
    .fetch_optional(ex)
    .await
}

pub async fn list_for_topic<'e>(
    ex: impl PgExecutor<'e>,
    topic_id: Uuid,
    limit: i64,
    offset: i64,
) -> sqlx::Result<Vec<PostRow>> {
    sqlx::query_as(
        "select p.id, p.topic_id, p.body, p.created_at, p.edited_at, \
            u.id as author_id, u.username as author_username, \
            u.display_name as author_display_name, u.role as author_role, \
            u.post_count as author_post_count, u.created_at as author_joined_at, \
            eu.id as editor_id, eu.username as editor_username, \
            eu.display_name as editor_display_name, eu.role as editor_role \
         from posts p \
         join users u on u.id = p.author_id \
         left join users eu on eu.id = p.edited_by \
         where p.topic_id = $1 order by p.id limit $2 offset $3",
    )
    .bind(topic_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(ex)
    .await
}

/// One post with its author and editor.
pub async fn get_one<'e>(ex: impl PgExecutor<'e>, id: Uuid) -> sqlx::Result<Option<PostRow>> {
    sqlx::query_as(
        "select p.id, p.topic_id, p.body, p.created_at, p.edited_at, \
            u.id as author_id, u.username as author_username, \
            u.display_name as author_display_name, u.role as author_role, \
            u.post_count as author_post_count, u.created_at as author_joined_at, \
            eu.id as editor_id, eu.username as editor_username, \
            eu.display_name as editor_display_name, eu.role as editor_role \
         from posts p \
         join users u on u.id = p.author_id \
         left join users eu on eu.id = p.edited_by \
         where p.id = $1",
    )
    .bind(id)
    .fetch_optional(ex)
    .await
}

/// 1-based position of a post within its topic (same order as the topic page).
pub async fn number_in_topic<'e>(
    ex: impl PgExecutor<'e>,
    topic_id: Uuid,
    post_id: Uuid,
) -> sqlx::Result<i64> {
    let row: (i64,) = sqlx::query_as("select count(*) from posts where topic_id = $1 and id <= $2")
        .bind(topic_id)
        .bind(post_id)
        .fetch_one(ex)
        .await?;
    Ok(row.0)
}

pub async fn count_for_topic<'e>(ex: impl PgExecutor<'e>, topic_id: Uuid) -> sqlx::Result<i64> {
    let row: (i64,) = sqlx::query_as("select count(*) from posts where topic_id = $1")
        .bind(topic_id)
        .fetch_one(ex)
        .await?;
    Ok(row.0)
}

/// Applies an edit: the caller has already locked the post with
/// [`lock_post`] in the same transaction. Writes the *old* body as a
/// revision, then updates the post with the new body.
pub async fn apply_edit(
    ex: &mut PgConnection,
    id: Uuid,
    revision_id: Uuid,
    old_body: &str,
    new_body: &str,
    editor_id: Uuid,
    now: DateTime<Utc>,
) -> sqlx::Result<()> {
    sqlx::query(
        "insert into post_revisions (id, post_id, body, edited_by, created_at) \
         values ($1, $2, $3, $4, $5)",
    )
    .bind(revision_id)
    .bind(id)
    .bind(old_body)
    .bind(editor_id)
    .bind(now)
    .execute(&mut *ex)
    .await?;
    sqlx::query("update posts set body = $2, edited_at = $3, edited_by = $4 where id = $1")
        .bind(id)
        .bind(new_body)
        .bind(now)
        .bind(editor_id)
        .execute(&mut *ex)
        .await?;
    Ok(())
}

pub async fn delete<'e>(ex: impl PgExecutor<'e>, id: Uuid) -> sqlx::Result<u64> {
    let result = sqlx::query("delete from posts where id = $1")
        .bind(id)
        .execute(ex)
        .await?;
    Ok(result.rows_affected())
}

pub async fn list_revisions<'e>(
    ex: impl PgExecutor<'e>,
    post_id: Uuid,
    limit: i64,
    offset: i64,
) -> sqlx::Result<Vec<PostRevisionRow>> {
    sqlx::query_as(
        "select r.id, r.body, r.created_at, u.id as editor_id, u.username as editor_username, \
            u.display_name as editor_display_name, u.role as editor_role \
         from post_revisions r left join users u on u.id = r.edited_by \
         where r.post_id = $1 order by r.id desc limit $2 offset $3",
    )
    .bind(post_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(ex)
    .await
}

pub async fn count_revisions<'e>(ex: impl PgExecutor<'e>, post_id: Uuid) -> sqlx::Result<i64> {
    let row: (i64,) = sqlx::query_as("select count(*) from post_revisions where post_id = $1")
        .bind(post_id)
        .fetch_one(ex)
        .await?;
    Ok(row.0)
}

pub async fn revision_body<'e>(
    ex: impl PgExecutor<'e>,
    post_id: Uuid,
    revision_id: Uuid,
) -> sqlx::Result<Option<String>> {
    let row: Option<(String,)> =
        sqlx::query_as("select body from post_revisions where id = $1 and post_id = $2")
            .bind(revision_id)
            .bind(post_id)
            .fetch_optional(ex)
            .await?;
    Ok(row.map(|r| r.0))
}

/// The author's most recent post `(created_at, body)`, for flood control and
/// the duplicate guard.
pub async fn latest_by_author<'e>(
    ex: impl PgExecutor<'e>,
    author_id: Uuid,
) -> sqlx::Result<Option<(DateTime<Utc>, String)>> {
    sqlx::query_as(
        "select created_at, body from posts where author_id = $1 order by created_at desc limit 1",
    )
    .bind(author_id)
    .fetch_optional(ex)
    .await
}

pub async fn count_all<'e>(ex: impl PgExecutor<'e>) -> sqlx::Result<i64> {
    let row: (i64,) = sqlx::query_as("select count(*) from posts")
        .fetch_one(ex)
        .await?;
    Ok(row.0)
}

/// Per-author post counts within a topic (to fix `users.post_count` when a
/// whole topic is deleted).
pub async fn author_counts_for_topic<'e>(
    ex: impl PgExecutor<'e>,
    topic_id: Uuid,
) -> sqlx::Result<Vec<(Uuid, i64)>> {
    sqlx::query_as("select author_id, count(*) from posts where topic_id = $1 group by author_id")
        .bind(topic_id)
        .fetch_all(ex)
        .await
}
