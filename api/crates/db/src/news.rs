use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgConnection, PgExecutor};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct NewsItemRow {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub published_at: DateTime<Utc>,
    pub author_id: Option<Uuid>,
    pub author_username: Option<String>,
    pub author_display_name: Option<String>,
    pub author_role: Option<String>,
}

const NEWS_SELECT: &str = "select n.id, n.title, n.slug, n.body, n.published_at, \
    u.id as author_id, u.username as author_username, u.display_name as author_display_name, \
    u.role as author_role \
    from news_items n left join users u on u.id = n.author_id";

pub async fn list<'e>(
    ex: impl PgExecutor<'e>,
    limit: i64,
    offset: i64,
) -> sqlx::Result<Vec<NewsItemRow>> {
    sqlx::query_as(const_format::concatcp!(
        NEWS_SELECT,
        " order by n.published_at desc, n.id desc limit $1 offset $2"
    ))
    .bind(limit)
    .bind(offset)
    .fetch_all(ex)
    .await
}

pub async fn count<'e>(ex: impl PgExecutor<'e>) -> sqlx::Result<i64> {
    let row: (i64,) = sqlx::query_as("select count(*) from news_items")
        .fetch_one(ex)
        .await?;
    Ok(row.0)
}

pub async fn get<'e>(ex: impl PgExecutor<'e>, id: Uuid) -> sqlx::Result<Option<NewsItemRow>> {
    sqlx::query_as(const_format::concatcp!(NEWS_SELECT, " where n.id = $1"))
        .bind(id)
        .fetch_optional(ex)
        .await
}

pub async fn get_for_update(
    ex: &mut PgConnection,
    id: Uuid,
) -> sqlx::Result<Option<(String, String)>> {
    sqlx::query_as("select title, body from news_items where id = $1 for update")
        .bind(id)
        .fetch_optional(ex)
        .await
}

pub async fn create<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    title: &str,
    slug: &str,
    body: &str,
    author_id: Uuid,
) -> sqlx::Result<()> {
    sqlx::query(
        "insert into news_items (id, title, slug, body, author_id) values ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(title)
    .bind(slug)
    .bind(body)
    .bind(author_id)
    .execute(ex)
    .await?;
    Ok(())
}

pub async fn update<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    title: &str,
    slug: &str,
    body: &str,
) -> sqlx::Result<()> {
    sqlx::query("update news_items set title = $2, slug = $3, body = $4 where id = $1")
        .bind(id)
        .bind(title)
        .bind(slug)
        .bind(body)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn delete<'e>(ex: impl PgExecutor<'e>, id: Uuid) -> sqlx::Result<u64> {
    let result = sqlx::query("delete from news_items where id = $1")
        .bind(id)
        .execute(ex)
        .await?;
    Ok(result.rows_affected())
}
