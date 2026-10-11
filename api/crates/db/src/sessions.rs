use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgExecutor};
use uuid::Uuid;

use crate::users::UserSummaryRow;

#[derive(Debug, Clone, FromRow)]
pub struct SessionAuthRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub csrf_token: String,
    pub remember: bool,
    pub expires_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    /// The user's *current* role, read on every request so role changes
    /// apply immediately.
    pub role: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct SessionListRow {
    pub id: Uuid,
    pub user_agent: Option<String>,
    pub remember: bool,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
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
    expires_at: DateTime<Utc>,
) -> sqlx::Result<()> {
    sqlx::query(
        "insert into sessions (id, user_id, token_hash, csrf_token, remember, user_agent, expires_at) \
         values ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(id)
    .bind(user_id)
    .bind(token_hash)
    .bind(csrf_token)
    .bind(remember)
    .bind(user_agent)
    .bind(expires_at)
    .execute(ex)
    .await?;
    Ok(())
}

pub async fn find_by_token_hash<'e>(
    ex: impl PgExecutor<'e>,
    token_hash: &str,
) -> sqlx::Result<Option<SessionAuthRow>> {
    sqlx::query_as(
        "select s.id, s.user_id, s.csrf_token, s.remember, s.expires_at, s.last_seen_at, u.role \
         from sessions s join users u on u.id = s.user_id \
         where s.token_hash = $1",
    )
    .bind(token_hash)
    .fetch_optional(ex)
    .await
}

/// Deletes the session with this token hash (login rotation); a no-op if
/// there is none.
pub async fn delete_by_token_hash<'e>(
    ex: impl PgExecutor<'e>,
    token_hash: &str,
) -> sqlx::Result<()> {
    sqlx::query("delete from sessions where token_hash = $1")
        .bind(token_hash)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn delete_by_id<'e>(ex: impl PgExecutor<'e>, id: Uuid) -> sqlx::Result<()> {
    sqlx::query("delete from sessions where id = $1")
        .bind(id)
        .execute(ex)
        .await?;
    Ok(())
}

/// Deletes one session only if it belongs to `user_id`; returns whether a
/// row was removed.
pub async fn delete_owned<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    user_id: Uuid,
) -> sqlx::Result<bool> {
    let result = sqlx::query("delete from sessions where id = $1 and user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(ex)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn delete_all_for_user_except<'e>(
    ex: impl PgExecutor<'e>,
    user_id: Uuid,
    keep_id: Option<Uuid>,
) -> sqlx::Result<u64> {
    let result = sqlx::query("delete from sessions where user_id = $1 and id is distinct from $2")
        .bind(user_id)
        .bind(keep_id)
        .execute(ex)
        .await?;
    Ok(result.rows_affected())
}

pub async fn list_for_user<'e>(
    ex: impl PgExecutor<'e>,
    user_id: Uuid,
) -> sqlx::Result<Vec<SessionListRow>> {
    sqlx::query_as(
        "select id, user_agent, remember, created_at, last_seen_at, expires_at \
         from sessions where user_id = $1 order by last_seen_at desc, id desc",
    )
    .bind(user_id)
    .fetch_all(ex)
    .await
}

/// Records activity. A single conditional UPDATE: it only writes when the
/// stored `last_seen_at` is at least `throttle` old, so a burst of requests
/// causes at most one write per interval and no read-modify-write race.
/// Non-remember sessions slide their idle expiry forward; remember sessions
/// keep their fixed 30-day expiry.
pub async fn touch<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    now: DateTime<Utc>,
    throttle: chrono::Duration,
    idle_expiry: DateTime<Utc>,
) -> sqlx::Result<bool> {
    let result = sqlx::query(
        "update sessions set last_seen_at = $2, \
         expires_at = case when remember then expires_at else $4 end \
         where id = $1 and last_seen_at <= $3",
    )
    .bind(id)
    .bind(now)
    .bind(now - throttle)
    .bind(idle_expiry)
    .execute(ex)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn purge_expired<'e>(ex: impl PgExecutor<'e>, now: DateTime<Utc>) -> sqlx::Result<u64> {
    let result = sqlx::query("delete from sessions where expires_at < $1")
        .bind(now)
        .execute(ex)
        .await?;
    Ok(result.rows_affected())
}

/// Number of distinct users with a session seen since `since`, excluding
/// users who opted out of the online list.
pub async fn count_online<'e>(ex: impl PgExecutor<'e>, since: DateTime<Utc>) -> sqlx::Result<i64> {
    let row: (i64,) = sqlx::query_as(
        "select count(distinct s.user_id) from sessions s join users u on u.id = s.user_id \
         where s.last_seen_at >= $1 and s.expires_at >= now() and u.show_online",
    )
    .bind(since)
    .fetch_one(ex)
    .await?;
    Ok(row.0)
}

pub async fn list_online<'e>(
    ex: impl PgExecutor<'e>,
    since: DateTime<Utc>,
    limit: i64,
) -> sqlx::Result<Vec<UserSummaryRow>> {
    sqlx::query_as(
        "select u.id, u.username, u.display_name, u.role from users u \
         where u.show_online and exists ( \
            select 1 from sessions s where s.user_id = u.id and s.last_seen_at >= $1 and s.expires_at >= now()) \
         order by lower(u.display_name), u.id limit $2",
    )
    .bind(since)
    .bind(limit)
    .fetch_all(ex)
    .await
}
