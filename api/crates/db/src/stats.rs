use serde::Serialize;
use sqlx::PgPool;
use utoipa::ToSchema;

use crate::{posts, structure, topics, users};

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub topics: i64,
    pub posts: i64,
    pub members: i64,
    pub boards: i64,
    pub newest_member: Option<String>,
    pub online_count: i64,
}

pub async fn get_stats(pool: &PgPool, online_count: i64) -> sqlx::Result<Stats> {
    let topics = topics::count_all(pool).await?;
    let posts = posts::count_all(pool).await?;
    let members = users::count_all(pool).await?;
    let boards = structure::count_boards(pool).await?;
    let newest_member = users::newest_member(pool).await?.map(|(name, _)| name);
    Ok(Stats {
        topics,
        posts,
        members,
        boards,
        newest_member,
        online_count,
    })
}
