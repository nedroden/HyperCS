use chrono::{DateTime, Utc};
use hypercs_domain::Role;
use sqlx::{FromRow, PgConnection, PgExecutor};
use uuid::Uuid;

/// Compact author/user summary joined into other rows' queries.
#[derive(Debug, Clone, FromRow)]
pub struct UserSummaryRow {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub role: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserAuthRow {
    pub id: Uuid,
    pub password_hash: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct MeRow {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub role: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
    pub about_me: Option<String>,
    pub language: String,
    pub timezone: String,
    pub notify_on_reply: bool,
    pub notify_on_quote: bool,
    pub show_online: bool,
    pub post_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct PublicProfileRow {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub location: Option<String>,
    pub website: Option<String>,
    pub about_me: Option<String>,
    pub post_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct AdminUserRow {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub role: String,
    pub post_count: i32,
    pub created_at: DateTime<Utc>,
}

const ME_SELECT: &str = "select id, username, display_name, email, role, first_name, last_name, \
    location, website, about_me, language, timezone, notify_on_reply, notify_on_quote, \
    show_online, post_count, created_at from users";

pub async fn create_user<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    username: &str,
    email: &str,
    password_hash: &str,
    role: Role,
    language: &str,
) -> sqlx::Result<()> {
    // The display name defaults to the username.
    sqlx::query(
        "insert into users (id, username, display_name, email, password_hash, role, language) \
         values ($1, $2, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .bind(role.as_db_str())
    .bind(language)
    .execute(ex)
    .await?;
    Ok(())
}

pub async fn find_auth_by_username<'e>(
    ex: impl PgExecutor<'e>,
    username: &str,
) -> sqlx::Result<Option<UserAuthRow>> {
    sqlx::query_as("select id, password_hash from users where lower(username) = lower($1)")
        .bind(username)
        .fetch_optional(ex)
        .await
}

/// Reads (and row-locks, inside a transaction) the account's own record.
pub async fn find_me_for_update(ex: &mut PgConnection, id: Uuid) -> sqlx::Result<Option<MeRow>> {
    sqlx::query_as(const_format::concatcp!(
        ME_SELECT,
        " where id = $1 for update"
    ))
    .bind(id)
    .fetch_optional(ex)
    .await
}

pub async fn find_me<'e>(ex: impl PgExecutor<'e>, id: Uuid) -> sqlx::Result<Option<MeRow>> {
    sqlx::query_as(const_format::concatcp!(ME_SELECT, " where id = $1"))
        .bind(id)
        .fetch_optional(ex)
        .await
}

pub async fn find_public_profile<'e>(
    ex: impl PgExecutor<'e>,
    username: &str,
) -> sqlx::Result<Option<PublicProfileRow>> {
    sqlx::query_as(
        "select id, username, display_name, role, location, website, about_me, post_count, created_at \
         from users where lower(username) = lower($1)",
    )
    .bind(username)
    .fetch_optional(ex)
    .await
}

pub async fn summary_by_id<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
) -> sqlx::Result<Option<UserSummaryRow>> {
    sqlx::query_as("select id, username, display_name, role from users where id = $1")
        .bind(id)
        .fetch_optional(ex)
        .await
}

pub async fn username_taken<'e>(ex: impl PgExecutor<'e>, username: &str) -> sqlx::Result<bool> {
    let row: (bool,) =
        sqlx::query_as("select exists(select 1 from users where lower(username) = lower($1))")
            .bind(username)
            .fetch_one(ex)
            .await?;
    Ok(row.0)
}

/// True if another account (not `except`) already uses the email.
pub async fn email_taken_by_other<'e>(
    ex: impl PgExecutor<'e>,
    email: &str,
    except: Option<Uuid>,
) -> sqlx::Result<bool> {
    let row: (bool,) = sqlx::query_as(
        "select exists(select 1 from users where lower(email) = lower($1) \
         and id is distinct from $2)",
    )
    .bind(email)
    .bind(except)
    .fetch_one(ex)
    .await?;
    Ok(row.0)
}

pub async fn any_admin_exists<'e>(ex: impl PgExecutor<'e>) -> sqlx::Result<bool> {
    let row: (bool,) = sqlx::query_as("select exists(select 1 from users where role = 'admin')")
        .fetch_one(ex)
        .await?;
    Ok(row.0)
}

pub async fn first_admin_id<'e>(ex: impl PgExecutor<'e>) -> sqlx::Result<Option<Uuid>> {
    let row: Option<(Uuid,)> =
        sqlx::query_as("select id from users where role = 'admin' order by created_at, id limit 1")
            .fetch_optional(ex)
            .await?;
    Ok(row.map(|r| r.0))
}

pub async fn update_profile<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    display_name: &str,
    location: Option<&str>,
    website: Option<&str>,
    about_me: Option<&str>,
) -> sqlx::Result<()> {
    sqlx::query(
        "update users set display_name = $2, location = $3, website = $4, about_me = $5 where id = $1",
    )
    .bind(id)
    .bind(display_name)
    .bind(location)
    .bind(website)
    .bind(about_me)
    .execute(ex)
    .await?;
    Ok(())
}

pub async fn update_account<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    first_name: Option<&str>,
    last_name: Option<&str>,
    email: &str,
) -> sqlx::Result<()> {
    sqlx::query("update users set first_name = $2, last_name = $3, email = $4 where id = $1")
        .bind(id)
        .bind(first_name)
        .bind(last_name)
        .bind(email)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn update_password_hash<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    password_hash: &str,
) -> sqlx::Result<()> {
    sqlx::query("update users set password_hash = $2 where id = $1")
        .bind(id)
        .bind(password_hash)
        .execute(ex)
        .await?;
    Ok(())
}

/// Locks the user row and returns the current password hash.
pub async fn password_hash_for_update(
    ex: &mut PgConnection,
    id: Uuid,
) -> sqlx::Result<Option<String>> {
    let row: Option<(String,)> =
        sqlx::query_as("select password_hash from users where id = $1 for update")
            .bind(id)
            .fetch_optional(ex)
            .await?;
    Ok(row.map(|r| r.0))
}

#[allow(clippy::too_many_arguments)]
pub async fn update_preferences<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    language: &str,
    timezone: &str,
    notify_on_reply: bool,
    notify_on_quote: bool,
    show_online: bool,
) -> sqlx::Result<()> {
    sqlx::query(
        "update users set language = $2, timezone = $3, notify_on_reply = $4, \
         notify_on_quote = $5, show_online = $6 where id = $1",
    )
    .bind(id)
    .bind(language)
    .bind(timezone)
    .bind(notify_on_reply)
    .bind(notify_on_quote)
    .bind(show_online)
    .execute(ex)
    .await?;
    Ok(())
}

pub async fn update_role<'e>(ex: impl PgExecutor<'e>, id: Uuid, role: Role) -> sqlx::Result<()> {
    sqlx::query("update users set role = $2 where id = $1")
        .bind(id)
        .bind(role.as_db_str())
        .execute(ex)
        .await?;
    Ok(())
}

/// Locks all admin rows (stable order) and returns their ids; used to make
/// "never remove the last admin" race-free.
pub async fn lock_admins(ex: &mut PgConnection) -> sqlx::Result<Vec<Uuid>> {
    let rows: Vec<(Uuid,)> =
        sqlx::query_as("select id from users where role = 'admin' order by id for update")
            .fetch_all(ex)
            .await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

/// Locks one user (looked up by case-insensitive username) and returns
/// `(id, role)`.
pub async fn lock_by_username(
    ex: &mut PgConnection,
    username: &str,
) -> sqlx::Result<Option<(Uuid, String)>> {
    sqlx::query_as("select id, role from users where lower(username) = lower($1) for update")
        .bind(username)
        .fetch_optional(ex)
        .await
}

pub async fn list_page<'e>(
    ex: impl PgExecutor<'e>,
    limit: i64,
    offset: i64,
) -> sqlx::Result<Vec<AdminUserRow>> {
    sqlx::query_as(
        "select id, username, display_name, email, role, post_count, created_at from users \
         order by created_at, id limit $1 offset $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(ex)
    .await
}

pub async fn count_all<'e>(ex: impl PgExecutor<'e>) -> sqlx::Result<i64> {
    let row: (i64,) = sqlx::query_as("select count(*) from users")
        .fetch_one(ex)
        .await?;
    Ok(row.0)
}

pub async fn newest_member<'e>(ex: impl PgExecutor<'e>) -> sqlx::Result<Option<UserSummaryRow>> {
    sqlx::query_as(
        "select id, username, display_name, role from users order by created_at desc, id desc limit 1",
    )
    .fetch_optional(ex)
    .await
}

pub async fn bump_post_count<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    delta: i32,
) -> sqlx::Result<()> {
    sqlx::query("update users set post_count = greatest(post_count + $2, 0) where id = $1")
        .bind(id)
        .bind(delta)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn created_at_by_id<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
) -> sqlx::Result<Option<DateTime<Utc>>> {
    let row: Option<(DateTime<Utc>,)> =
        sqlx::query_as("select created_at from users where id = $1")
            .bind(id)
            .fetch_optional(ex)
            .await?;
    Ok(row.map(|r| r.0))
}

/// Serializes one user's posting (flood control + duplicate guard) without
/// touching the user row, so it cannot deadlock with counter updates.
pub async fn lock_posting(ex: &mut PgConnection, user_id: Uuid) -> sqlx::Result<()> {
    sqlx::query("select pg_advisory_xact_lock(hashtextextended(($1::uuid)::text, 0))")
        .bind(user_id)
        .execute(ex)
        .await?;
    Ok(())
}
