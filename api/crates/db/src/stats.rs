use sqlx::PgExecutor;

use crate::{posts, structure, topics, users};

#[derive(Debug, Clone)]
pub struct Totals {
    pub topics: i64,
    pub posts: i64,
    pub members: i64,
    pub boards: i64,
}

pub async fn totals<'e>(ex: impl PgExecutor<'e> + Copy) -> sqlx::Result<Totals> {
    Ok(Totals {
        topics: topics::count_all(ex).await?,
        posts: posts::count_all(ex).await?,
        members: users::count_all(ex).await?,
        boards: structure::count_boards(ex).await?,
    })
}
