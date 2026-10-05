use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, PgExecutor};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct SessionAuthRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub csrf_token: String,
    pub expires_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub role: String,
}

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionListRow {
    pub id: Uuid,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

#[allow(clippy::too_many_arguments)]
pub async fn create_session<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    user_id: Uuid,
    token_hash: &str,
    csrf_token: &str,
    remember: bool,
    user_agent: Option<&str>,
    ip: Option<&str>,
    expires_at: DateTime<Utc>,
) -> sqlx::Result<()> {
    sqlx::query(
        "insert into sessions (id, user_id, token_hash, csrf_token, remember, user_agent, ip, expires_at) \
         values ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(id)
    .bind(user_id)
    .bind(token_hash)
    .bind(csrf_token)
    .bind(remember)
    .bind(user_agent)
    .bind(ip)
    .bind(expires_at)
    .execute(ex)
    .await?;
    Ok(())
}

pub async fn find_by_token_hash<'e>(
    ex: impl PgExecutor<'e>,
    token_hash: &str,
) -> sqlx::Result<Option<SessionAuthRow>> {
    sqlx::query_as::<_, SessionAuthRow>(
        "select s.id, s.user_id, s.csrf_token, s.expires_at, s.last_seen_at, u.role \
         from sessions s join users u on u.id = s.user_id \
         where s.token_hash = $1",
    )
    .bind(token_hash)
    .fetch_optional(ex)
    .await
}

pub async fn delete_by_id<'e>(ex: impl PgExecutor<'e>, id: Uuid) -> sqlx::Result<()> {
    sqlx::query("delete from sessions where id = $1")
        .bind(id)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn delete_all_for_user_except<'e>(
    ex: impl PgExecutor<'e>,
    user_id: Uuid,
    keep_id: Option<Uuid>,
) -> sqlx::Result<()> {
    sqlx::query("delete from sessions where user_id = $1 and id is distinct from $2")
        .bind(user_id)
        .bind(keep_id)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn list_for_user<'e>(
    ex: impl PgExecutor<'e>,
    user_id: Uuid,
) -> sqlx::Result<Vec<SessionListRow>> {
    sqlx::query_as::<_, SessionListRow>(
        "select id, user_agent, created_at, last_seen_at from sessions where user_id = $1 order by last_seen_at desc",
    )
    .bind(user_id)
    .fetch_all(ex)
    .await
}

pub async fn touch_last_seen<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    now: DateTime<Utc>,
) -> sqlx::Result<()> {
    sqlx::query("update sessions set last_seen_at = $2 where id = $1")
        .bind(id)
        .bind(now)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn purge_expired<'e>(ex: impl PgExecutor<'e>, now: DateTime<Utc>) -> sqlx::Result<u64> {
    let result = sqlx::query("delete from sessions where expires_at < $1")
        .bind(now)
        .execute(ex)
        .await?;
    Ok(result.rows_affected())
}

pub async fn count_online<'e>(ex: impl PgExecutor<'e>, since: DateTime<Utc>) -> sqlx::Result<i64> {
    let row: (i64,) = sqlx::query_as(
        "select count(distinct s.user_id) from sessions s join users u on u.id = s.user_id \
         where s.last_seen_at >= $1 and u.show_online",
    )
    .bind(since)
    .fetch_one(ex)
    .await?;
    Ok(row.0)
}

pub async fn csrf_token_for_session<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
) -> sqlx::Result<Option<String>> {
    let row: Option<(String,)> = sqlx::query_as("select csrf_token from sessions where id = $1")
        .bind(id)
        .fetch_optional(ex)
        .await?;
    Ok(row.map(|r| r.0))
}
