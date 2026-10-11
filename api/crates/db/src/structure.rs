use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgConnection, PgExecutor};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct CategoryRow {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub position: i32,
    pub admin_only: bool,
}

/// A board for the board index, with its last post joined in (all `lp_*`
/// columns are NULL for a board without topics).
#[derive(Debug, Clone, FromRow)]
pub struct BoardIndexRow {
    pub id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub description: String,
    pub position: i32,
    pub admin_only: bool,
    pub topic_count: i32,
    pub post_count: i32,
    pub lp_post_id: Option<Uuid>,
    pub lp_topic_id: Option<Uuid>,
    pub lp_topic_title: Option<String>,
    pub lp_created_at: Option<DateTime<Utc>>,
    pub lp_author_id: Option<Uuid>,
    pub lp_author_username: Option<String>,
    pub lp_author_display_name: Option<String>,
    pub lp_author_role: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct BoardRow {
    pub id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub description: String,
    pub position: i32,
    pub admin_only: bool,
}

/// A board with its category; `admin_only` is the *effective* flag (the
/// board's own OR its category's).
#[derive(Debug, Clone, FromRow)]
pub struct BoardDetailRow {
    pub id: Uuid,
    pub category_id: Uuid,
    pub category_name: String,
    pub name: String,
    pub description: String,
    pub admin_only: bool,
    pub topic_count: i32,
    pub post_count: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct LastPostRow {
    pub post_id: Uuid,
    pub topic_id: Uuid,
    pub topic_title: String,
    pub created_at: DateTime<Utc>,
    pub author_id: Uuid,
    pub author_username: String,
    pub author_display_name: String,
    pub author_role: String,
}

pub async fn create_category<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    name: &str,
    description: &str,
    admin_only: bool,
    position: i32,
) -> sqlx::Result<()> {
    sqlx::query(
        "insert into categories (id, name, description, admin_only, position) values ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .bind(admin_only)
    .bind(position)
    .execute(ex)
    .await?;
    Ok(())
}

pub async fn next_category_position<'e>(ex: impl PgExecutor<'e>) -> sqlx::Result<i32> {
    let row: (Option<i32>,) = sqlx::query_as("select max(position) from categories")
        .fetch_one(ex)
        .await?;
    Ok(row.0.map_or(0, |p| p + 1))
}

pub async fn get_category_for_update(
    ex: &mut PgConnection,
    id: Uuid,
) -> sqlx::Result<Option<CategoryRow>> {
    sqlx::query_as(
        "select id, name, description, position, admin_only from categories where id = $1 for update",
    )
    .bind(id)
    .fetch_optional(ex)
    .await
}

pub async fn category_exists<'e>(ex: impl PgExecutor<'e>, id: Uuid) -> sqlx::Result<bool> {
    let row: (bool,) = sqlx::query_as("select exists(select 1 from categories where id = $1)")
        .bind(id)
        .fetch_one(ex)
        .await?;
    Ok(row.0)
}

pub async fn update_category<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    name: &str,
    description: &str,
    admin_only: bool,
) -> sqlx::Result<()> {
    sqlx::query("update categories set name = $2, description = $3, admin_only = $4 where id = $1")
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(admin_only)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn delete_category<'e>(ex: impl PgExecutor<'e>, id: Uuid) -> sqlx::Result<u64> {
    let result = sqlx::query("delete from categories where id = $1")
        .bind(id)
        .execute(ex)
        .await?;
    Ok(result.rows_affected())
}

pub async fn list_categories_ordered_for_update(
    ex: &mut PgConnection,
) -> sqlx::Result<Vec<CategoryRow>> {
    sqlx::query_as(
        "select id, name, description, position, admin_only from categories \
         order by position, id for update",
    )
    .fetch_all(ex)
    .await
}

pub async fn set_category_position<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    position: i32,
) -> sqlx::Result<()> {
    sqlx::query("update categories set position = $2 where id = $1")
        .bind(id)
        .bind(position)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn create_board<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    category_id: Uuid,
    name: &str,
    description: &str,
    admin_only: bool,
    position: i32,
) -> sqlx::Result<()> {
    sqlx::query(
        "insert into boards (id, category_id, name, description, admin_only, position) \
         values ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(category_id)
    .bind(name)
    .bind(description)
    .bind(admin_only)
    .bind(position)
    .execute(ex)
    .await?;
    Ok(())
}

pub async fn next_board_position<'e>(
    ex: impl PgExecutor<'e>,
    category_id: Uuid,
) -> sqlx::Result<i32> {
    let row: (Option<i32>,) =
        sqlx::query_as("select max(position) from boards where category_id = $1")
            .bind(category_id)
            .fetch_one(ex)
            .await?;
    Ok(row.0.map_or(0, |p| p + 1))
}

pub async fn get_board_for_update(
    ex: &mut PgConnection,
    id: Uuid,
) -> sqlx::Result<Option<BoardRow>> {
    sqlx::query_as(
        "select id, category_id, name, description, position, admin_only from boards \
         where id = $1 for update",
    )
    .bind(id)
    .fetch_optional(ex)
    .await
}

pub async fn update_board<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    name: &str,
    description: &str,
    admin_only: bool,
) -> sqlx::Result<()> {
    sqlx::query("update boards set name = $2, description = $3, admin_only = $4 where id = $1")
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(admin_only)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn delete_board<'e>(ex: impl PgExecutor<'e>, id: Uuid) -> sqlx::Result<u64> {
    let result = sqlx::query("delete from boards where id = $1")
        .bind(id)
        .execute(ex)
        .await?;
    Ok(result.rows_affected())
}

pub async fn list_boards_ordered_for_update(
    ex: &mut PgConnection,
    category_id: Uuid,
) -> sqlx::Result<Vec<BoardRow>> {
    sqlx::query_as(
        "select id, category_id, name, description, position, admin_only from boards \
         where category_id = $1 order by position, id for update",
    )
    .bind(category_id)
    .fetch_all(ex)
    .await
}

pub async fn set_board_position<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
    position: i32,
) -> sqlx::Result<()> {
    sqlx::query("update boards set position = $2 where id = $1")
        .bind(id)
        .bind(position)
        .execute(ex)
        .await?;
    Ok(())
}

const BOARD_DETAIL_SELECT: &str = "select b.id, b.category_id, c.name as category_name, b.name, \
    b.description, (b.admin_only or c.admin_only) as admin_only, b.topic_count, b.post_count \
    from boards b join categories c on c.id = b.category_id";

pub async fn get_board_detail<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
) -> sqlx::Result<Option<BoardDetailRow>> {
    sqlx::query_as(const_format::concatcp!(
        BOARD_DETAIL_SELECT,
        " where b.id = $1"
    ))
    .bind(id)
    .fetch_optional(ex)
    .await
}

/// Row-locks the board (not the category) and returns it with the effective
/// admin-only flag. The lock serializes counter updates and prevents the
/// board from being deleted underneath an insert.
pub async fn lock_board_detail(
    ex: &mut PgConnection,
    id: Uuid,
) -> sqlx::Result<Option<BoardDetailRow>> {
    sqlx::query_as(const_format::concatcp!(
        BOARD_DETAIL_SELECT,
        " where b.id = $1 for update of b"
    ))
    .bind(id)
    .fetch_optional(ex)
    .await
}

pub async fn list_categories<'e>(
    ex: impl PgExecutor<'e>,
    include_admin_only: bool,
) -> sqlx::Result<Vec<CategoryRow>> {
    sqlx::query_as(
        "select id, name, description, position, admin_only from categories \
         where ($1 or not admin_only) order by position, id",
    )
    .bind(include_admin_only)
    .fetch_all(ex)
    .await
}

/// All boards of all (visible) categories with their last post joined in:
/// one query for the whole board index.
pub async fn list_board_index_rows<'e>(
    ex: impl PgExecutor<'e>,
    include_admin_only: bool,
) -> sqlx::Result<Vec<BoardIndexRow>> {
    sqlx::query_as(
        "select b.id, b.category_id, b.name, b.description, b.position, b.admin_only, \
            b.topic_count, b.post_count, \
            p.id as lp_post_id, t.id as lp_topic_id, t.title as lp_topic_title, \
            p.created_at as lp_created_at, u.id as lp_author_id, u.username as lp_author_username, \
            u.display_name as lp_author_display_name, u.role as lp_author_role \
         from boards b \
         join categories c on c.id = b.category_id \
         left join posts p on p.id = b.last_post_id \
         left join topics t on t.id = p.topic_id \
         left join users u on u.id = p.author_id \
         where ($1 or not (b.admin_only or c.admin_only)) \
         order by c.position, c.id, b.position, b.id",
    )
    .bind(include_admin_only)
    .fetch_all(ex)
    .await
}

/// The most recent post in boards visible to everyone (used by statistics;
/// never leaks admin-only boards).
pub async fn latest_public_post<'e>(ex: impl PgExecutor<'e>) -> sqlx::Result<Option<LastPostRow>> {
    sqlx::query_as(
        "select p.id as post_id, t.id as topic_id, t.title as topic_title, p.created_at, \
            u.id as author_id, u.username as author_username, \
            u.display_name as author_display_name, u.role as author_role \
         from posts p \
         join topics t on t.id = p.topic_id \
         join boards b on b.id = t.board_id \
         join categories c on c.id = b.category_id \
         join users u on u.id = p.author_id \
         where not (b.admin_only or c.admin_only) \
         order by p.id desc limit 1",
    )
    .fetch_optional(ex)
    .await
}

pub async fn bump_board_counts<'e>(
    ex: impl PgExecutor<'e>,
    board_id: Uuid,
    topic_delta: i32,
    post_delta: i32,
) -> sqlx::Result<()> {
    sqlx::query(
        "update boards set topic_count = greatest(topic_count + $2, 0), \
         post_count = greatest(post_count + $3, 0) where id = $1",
    )
    .bind(board_id)
    .bind(topic_delta)
    .bind(post_delta)
    .execute(ex)
    .await?;
    Ok(())
}

pub async fn set_board_last_post<'e>(
    ex: impl PgExecutor<'e>,
    board_id: Uuid,
    last_post_id: Option<Uuid>,
    last_post_at: Option<DateTime<Utc>>,
) -> sqlx::Result<()> {
    sqlx::query("update boards set last_post_id = $2, last_post_at = $3 where id = $1")
        .bind(board_id)
        .bind(last_post_id)
        .bind(last_post_at)
        .execute(ex)
        .await?;
    Ok(())
}

/// Points the board's last post at the most recently active remaining topic
/// (or clears it when the board has no topics). Uses the denormalized topic
/// pointers, so the topic's own aggregates must be fresh first.
pub async fn refresh_board_last_post<'e>(
    ex: impl PgExecutor<'e>,
    board_id: Uuid,
) -> sqlx::Result<()> {
    sqlx::query(
        "update boards set \
            last_post_id = (select last_post_id from topics where board_id = $1 \
                            order by last_post_at desc, id desc limit 1), \
            last_post_at = (select last_post_at from topics where board_id = $1 \
                            order by last_post_at desc, id desc limit 1) \
         where id = $1",
    )
    .bind(board_id)
    .execute(ex)
    .await?;
    Ok(())
}

/// Subtracts the posts of a board (about to be deleted with it) from their
/// authors' `post_count`.
pub async fn decrement_author_counts_for_board<'e>(
    ex: impl PgExecutor<'e>,
    board_id: Uuid,
) -> sqlx::Result<()> {
    sqlx::query(
        "update users u set post_count = greatest(u.post_count - c.n, 0) \
         from (select p.author_id, count(*)::int as n from posts p \
               join topics t on t.id = p.topic_id where t.board_id = $1 group by p.author_id) c \
         where u.id = c.author_id",
    )
    .bind(board_id)
    .execute(ex)
    .await?;
    Ok(())
}

/// Same for every board of a category about to be deleted.
pub async fn decrement_author_counts_for_category<'e>(
    ex: impl PgExecutor<'e>,
    category_id: Uuid,
) -> sqlx::Result<()> {
    sqlx::query(
        "update users u set post_count = greatest(u.post_count - c.n, 0) \
         from (select p.author_id, count(*)::int as n from posts p \
               join topics t on t.id = p.topic_id join boards b on b.id = t.board_id \
               where b.category_id = $1 group by p.author_id) c \
         where u.id = c.author_id",
    )
    .bind(category_id)
    .execute(ex)
    .await?;
    Ok(())
}

pub async fn count_boards<'e>(ex: impl PgExecutor<'e>) -> sqlx::Result<i64> {
    let row: (i64,) = sqlx::query_as("select count(*) from boards")
        .fetch_one(ex)
        .await?;
    Ok(row.0)
}

pub async fn count_categories<'e>(ex: impl PgExecutor<'e>) -> sqlx::Result<i64> {
    let row: (i64,) = sqlx::query_as("select count(*) from categories")
        .fetch_one(ex)
        .await?;
    Ok(row.0)
}
