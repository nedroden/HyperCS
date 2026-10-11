use hypercs_db::PgPool;
use hypercs_domain::constants::{ONLINE_USERS_LIMIT, ONLINE_WINDOW_MINUTES};

use crate::{
    dto::{Stats, UserSummary},
    error::ApiResult,
    services::view,
};

pub async fn get_stats(db: &PgPool) -> ApiResult<Stats> {
    let since = chrono::Utc::now() - chrono::Duration::minutes(ONLINE_WINDOW_MINUTES);
    let totals = hypercs_db::stats::totals(db).await?;
    let newest_member = hypercs_db::users::newest_member(db).await?;
    let latest_post = hypercs_db::structure::latest_public_post(db).await?;
    let online_count = hypercs_db::sessions::count_online(db, since).await?;
    let online_users = hypercs_db::sessions::list_online(db, since, ONLINE_USERS_LIMIT).await?;
    Ok(Stats {
        topics: totals.topics,
        posts: totals.posts,
        members: totals.members,
        boards: totals.boards,
        newest_member: newest_member.map(UserSummary::from),
        latest_post: latest_post.map(view::last_post),
        online_count,
        online_users: online_users.into_iter().map(UserSummary::from).collect(),
    })
}
