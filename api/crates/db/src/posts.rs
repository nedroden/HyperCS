use chrono::{DateTime, Utc};
use const_format::concatcp;
use serde::Serialize;
use sqlx::{FromRow, PgExecutor};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PostRow {
    pub id: Uuid,
    pub topic_id: Uuid,
    pub author_id: Uuid,
    pub author_username: String,
    pub author_display_name: String,
    pub author_role: String,
    pub author_post_count: i32,
    pub author_joined_at: DateTime<Utc>,
    pub body: String,
    pub edited_at: Option<DateTime<Utc>>,
    pub edited_by_display_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct PostOwnerRow {
    pub topic_id: Uuid,
    pub author_id: Uuid,
    pub body: String,
}

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PostRevisionRow {
    pub id: Uuid,
    pub body: String,
    pub edited_by_display_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

const POST_SELECT: &str = "select p.id, p.topic_id, p.author_id, u.username as author_username, \
    u.display_name as author_display_name, u.role as author_role, u.post_count as author_post_count, \
    u.created_at as author_joined_at, p.body, p.edited_at, eu.display_name as edited_by_display_name, \
    p.created_at \
    from posts p join users u on u.id = p.author_id \
    left join users eu on eu.id = p.edited_by";

pub async fn create_post<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    topic_id: Uuid,
    author_id: Uuid,
    body: &str,
) -> sqlx::Result<()> {
    sqlx::query("insert into posts (id, topic_id, author_id, body) values ($1, $2, $3, $4)")
        .bind(id)
        .bind(topic_id)
        .bind(author_id)
        .bind(body)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn get<'e>(ex: impl PgExecutor<'e>, id: Uuid) -> sqlx::Result<Option<PostRow>> {
    sqlx::query_as::<_, PostRow>(concatcp!(POST_SELECT, " where p.id = $1"))
        .bind(id)
        .fetch_optional(ex)
        .await
}

pub async fn get_owner<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
) -> sqlx::Result<Option<PostOwnerRow>> {
    sqlx::query_as::<_, PostOwnerRow>(
        "select topic_id, author_id, body from posts where id = $1 for update",
    )
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
    sqlx::query_as::<_, PostRow>(concatcp!(
        POST_SELECT,
        " where p.topic_id = $1 order by p.created_at limit $2 offset $3"
    ))
    .bind(topic_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(ex)
    .await
}

pub async fn count_for_topic<'e>(ex: impl PgExecutor<'e>, topic_id: Uuid) -> sqlx::Result<i64> {
    let row: (i64,) = sqlx::query_as("select count(*) from posts where topic_id = $1")
        .bind(topic_id)
        .fetch_one(ex)
        .await?;
    Ok(row.0)
}

pub async fn first_post_id_for_topic<'e>(
    ex: impl PgExecutor<'e>,
    topic_id: Uuid,
) -> sqlx::Result<Option<Uuid>> {
    let row: Option<(Uuid,)> =
        sqlx::query_as("select id from posts where topic_id = $1 order by created_at limit 1")
            .bind(topic_id)
            .fetch_optional(ex)
            .await?;
    Ok(row.map(|r| r.0))
}

/// Applies an edit: caller must have already locked the row with
/// [`get_owner`] inside the same transaction. Writes the *old* body as a
/// revision, then updates the post with the new body.
pub async fn apply_edit(
    ex: &mut sqlx::PgConnection,
    id: Uuid,
    revision_id: Uuid,
    old_body: &str,
    new_body: &str,
    edited_by: Uuid,
    now: DateTime<Utc>,
) -> sqlx::Result<()> {
    sqlx::query(
        "insert into post_revisions (id, post_id, body, edited_by) values ($1, $2, $3, $4)",
    )
    .bind(revision_id)
    .bind(id)
    .bind(old_body)
    .bind(edited_by)
    .execute(&mut *ex)
    .await?;
    sqlx::query("update posts set body = $2, edited_at = $3, edited_by = $4 where id = $1")
        .bind(id)
        .bind(new_body)
        .bind(now)
        .bind(edited_by)
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
) -> sqlx::Result<Vec<PostRevisionRow>> {
    sqlx::query_as::<_, PostRevisionRow>(
        "select r.id, r.body, u.display_name as edited_by_display_name, r.created_at \
         from post_revisions r left join users u on u.id = r.edited_by \
         where r.post_id = $1 order by r.created_at desc",
    )
    .bind(post_id)
    .fetch_all(ex)
    .await
}

pub async fn last_post_time_for_author<'e>(
    ex: impl PgExecutor<'e>,
    author_id: Uuid,
) -> sqlx::Result<Option<DateTime<Utc>>> {
    let row: Option<(DateTime<Utc>,)> = sqlx::query_as(
        "select created_at from posts where author_id = $1 order by created_at desc limit 1",
    )
    .bind(author_id)
    .fetch_optional(ex)
    .await?;
    Ok(row.map(|r| r.0))
}

pub async fn last_post_body_for_author<'e>(
    ex: impl PgExecutor<'e>,
    author_id: Uuid,
) -> sqlx::Result<Option<String>> {
    let row: Option<(String,)> = sqlx::query_as(
        "select body from posts where author_id = $1 order by created_at desc limit 1",
    )
    .bind(author_id)
    .fetch_optional(ex)
    .await?;
    Ok(row.map(|r| r.0))
}

pub async fn count_all<'e>(ex: impl PgExecutor<'e>) -> sqlx::Result<i64> {
    let row: (i64,) = sqlx::query_as("select count(*) from posts")
        .fetch_one(ex)
        .await?;
    Ok(row.0)
}

/// Per-author post counts within a topic (used to fix up `users.post_count`
/// when a whole topic is deleted).
pub async fn author_counts_for_topic<'e>(
    ex: impl PgExecutor<'e>,
    topic_id: Uuid,
) -> sqlx::Result<Vec<(Uuid, i64)>> {
    sqlx::query_as("select author_id, count(*) from posts where topic_id = $1 group by author_id")
        .bind(topic_id)
        .fetch_all(ex)
        .await
}
