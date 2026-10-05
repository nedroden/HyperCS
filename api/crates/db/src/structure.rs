use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, PgExecutor};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CategoryRow {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub position: i32,
    pub admin_only: bool,
}

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BoardRow {
    pub id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub description: String,
    pub position: i32,
    pub admin_only: bool,
    pub topic_count: i32,
    pub post_count: i32,
}

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LastPostRow {
    pub post_id: Uuid,
    pub topic_id: Uuid,
    pub topic_title: String,
    pub author_id: Uuid,
    pub author_username: String,
    pub author_display_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BoardIndexItem {
    #[serde(flatten)]
    pub board: BoardRow,
    pub last_post: Option<LastPostRow>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CategoryIndexItem {
    #[serde(flatten)]
    pub category: CategoryRow,
    pub boards: Vec<BoardIndexItem>,
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
    Ok(row.0.map(|p| p + 1).unwrap_or(0))
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

pub async fn list_categories_ordered<'e>(
    ex: impl PgExecutor<'e>,
) -> sqlx::Result<Vec<CategoryRow>> {
    sqlx::query_as::<_, CategoryRow>(
        "select id, name, description, position, admin_only from categories order by position",
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
    Ok(row.0.map(|p| p + 1).unwrap_or(0))
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

pub async fn list_boards_ordered<'e>(
    ex: impl PgExecutor<'e>,
    category_id: Uuid,
) -> sqlx::Result<Vec<BoardRow>> {
    sqlx::query_as::<_, BoardRow>(
        "select id, category_id, name, description, position, admin_only, topic_count, post_count \
         from boards where category_id = $1 order by position",
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

pub async fn get_board_detail<'e>(
    ex: impl PgExecutor<'e>,
    id: Uuid,
) -> sqlx::Result<Option<BoardDetailRow>> {
    sqlx::query_as::<_, BoardDetailRow>(
        "select b.id, b.category_id, c.name as category_name, b.name, b.description, b.admin_only, \
         b.topic_count, b.post_count \
         from boards b join categories c on c.id = b.category_id where b.id = $1",
    )
    .bind(id)
    .fetch_optional(ex)
    .await
}

pub async fn board_admin_only<'e>(ex: impl PgExecutor<'e>, id: Uuid) -> sqlx::Result<Option<bool>> {
    let row: Option<(bool,)> = sqlx::query_as("select admin_only from boards where id = $1")
        .bind(id)
        .fetch_optional(ex)
        .await?;
    Ok(row.map(|r| r.0))
}

const LAST_POST_JOIN: &str = "select p.id as post_id, t.id as topic_id, t.title as topic_title, \
    u.id as author_id, u.username as author_username, u.display_name as author_display_name, p.created_at \
    from posts p join topics t on t.id = p.topic_id join users u on u.id = p.author_id where p.id = $1";

pub async fn list_board_index<'e>(
    ex: impl PgExecutor<'e> + Copy,
    include_admin_only: bool,
) -> sqlx::Result<Vec<CategoryIndexItem>> {
    let categories: Vec<CategoryRow> = if include_admin_only {
        sqlx::query_as(
            "select id, name, description, position, admin_only from categories order by position",
        )
        .fetch_all(ex)
        .await?
    } else {
        sqlx::query_as(
            "select id, name, description, position, admin_only from categories \
             where not admin_only order by position",
        )
        .fetch_all(ex)
        .await?
    };

    let mut result = Vec::with_capacity(categories.len());
    for category in categories {
        let boards: Vec<BoardRow> = if include_admin_only {
            sqlx::query_as(
                "select id, category_id, name, description, position, admin_only, topic_count, post_count \
                 from boards where category_id = $1 order by position",
            )
            .bind(category.id)
            .fetch_all(ex)
            .await?
        } else {
            sqlx::query_as(
                "select id, category_id, name, description, position, admin_only, topic_count, post_count \
                 from boards where category_id = $1 and not admin_only order by position",
            )
            .bind(category.id)
            .fetch_all(ex)
            .await?
        };

        let mut board_items = Vec::with_capacity(boards.len());
        for board in boards {
            let last_post: Option<LastPostRow> = sqlx::query_as(
                "select p.id as post_id, t.id as topic_id, t.title as topic_title, \
                 u.id as author_id, u.username as author_username, u.display_name as author_display_name, p.created_at \
                 from boards b join posts p on p.id = b.last_post_id \
                 join topics t on t.id = p.topic_id join users u on u.id = p.author_id where b.id = $1",
            )
            .bind(board.id)
            .fetch_optional(ex)
            .await?;
            board_items.push(BoardIndexItem { board, last_post });
        }
        result.push(CategoryIndexItem {
            category,
            boards: board_items,
        });
    }
    Ok(result)
}

/// Fetches the last-post summary for an arbitrary post id (used after moving/
/// deleting topics to recompute a board's `last_post_id`).
pub async fn last_post_summary<'e>(
    ex: impl PgExecutor<'e>,
    post_id: Uuid,
) -> sqlx::Result<Option<LastPostRow>> {
    sqlx::query_as::<_, LastPostRow>(LAST_POST_JOIN)
        .bind(post_id)
        .fetch_optional(ex)
        .await
}

pub async fn bump_board_counts<'e>(
    ex: impl PgExecutor<'e>,
    board_id: Uuid,
    topic_delta: i32,
    post_delta: i32,
) -> sqlx::Result<()> {
    sqlx::query("update boards set topic_count = topic_count + $2, post_count = post_count + $3 where id = $1")
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
) -> sqlx::Result<()> {
    sqlx::query("update boards set last_post_id = $2 where id = $1")
        .bind(board_id)
        .bind(last_post_id)
        .execute(ex)
        .await?;
    Ok(())
}

/// Recomputes a board's `last_post_id` from its most recent remaining post
/// (used after a delete/move changes which post was last). Used by the
/// `recount` CLI and by delete/move services as a safety net.
pub async fn recompute_board_last_post<'e>(
    ex: impl PgExecutor<'e>,
    board_id: Uuid,
) -> sqlx::Result<()> {
    sqlx::query(
        "update boards set last_post_id = ( \
            select p.id from posts p join topics t on t.id = p.topic_id \
            where t.board_id = boards.id order by p.created_at desc limit 1 \
         ) where id = $1",
    )
    .bind(board_id)
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

pub async fn all_board_ids<'e>(ex: impl PgExecutor<'e>) -> sqlx::Result<Vec<Uuid>> {
    let rows: Vec<(Uuid,)> = sqlx::query_as("select id from boards")
        .fetch_all(ex)
        .await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

/// Recomputes a board's `topic_count`/`post_count` from its topics/posts
/// (used by the `recount` CLI).
pub async fn recompute_board_aggregates(
    ex: &mut sqlx::PgConnection,
    board_id: Uuid,
) -> sqlx::Result<()> {
    sqlx::query(
        "update boards set topic_count = (select count(*) from topics where board_id = $1), \
         post_count = (select count(*) from posts p join topics t on t.id = p.topic_id where t.board_id = $1) \
         where id = $1",
    )
    .bind(board_id)
    .execute(&mut *ex)
    .await?;
    recompute_board_last_post(&mut *ex, board_id).await
}
