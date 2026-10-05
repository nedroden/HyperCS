use hypercs_db::PgPool;
use hypercs_domain::DomainResult;
use hypercs_domain::constants::ONLINE_WINDOW_MINUTES;

pub async fn get_stats(db: &PgPool) -> DomainResult<hypercs_db::stats::Stats> {
    let since = chrono::Utc::now() - chrono::Duration::minutes(ONLINE_WINDOW_MINUTES);
    let online = hypercs_db::sessions::count_online(db, since)
        .await
        .map_err(|_| hypercs_domain::DomainError::Internal)?;
    hypercs_db::stats::get_stats(db, online)
        .await
        .map_err(|_| hypercs_domain::DomainError::Internal)
}
