use std::time::Duration;

use hypercs_db::PgPool;

/// Periodically purges expired sessions (in-process tokio task). The
/// throttled `last_seen_at` update used for the users-online list happens
/// inline in the auth extractor instead: it only needs to run when a session
/// is actually used.
pub fn spawn_session_purge(db: PgPool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(300));
        loop {
            interval.tick().await;
            match hypercs_db::sessions::purge_expired(&db, chrono::Utc::now()).await {
                Ok(count) if count > 0 => tracing::info!(count, "purged expired sessions"),
                Ok(_) => {}
                Err(error) => tracing::warn!(%error, "failed to purge expired sessions"),
            }
        }
    });
}
