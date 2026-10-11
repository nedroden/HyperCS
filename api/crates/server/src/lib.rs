//! The HyperCS server as a library: configuration, wiring and the tasks the
//! binary (`main.rs`) and the integration tests share.

pub mod config;
pub mod housekeeping;
pub mod recount;
pub mod seed;

use hypercs_db::PgPool;
use sqlx::postgres::PgPoolOptions;

/// Creates the connection pool (lazily: the first query connects, so the
/// process can start before the database is reachable).
pub fn connect(config: &config::Config) -> anyhow::Result<PgPool> {
    Ok(PgPoolOptions::new()
        .max_connections(10)
        .connect_lazy(&config.database_url)?)
}

/// Runs pending migrations, retrying while the database is still starting.
pub async fn run_migrations(db: &PgPool) -> anyhow::Result<()> {
    let mut attempts = 0;
    loop {
        match hypercs_db::MIGRATOR.run(db).await {
            Ok(()) => return Ok(()),
            Err(e) if attempts < 30 => {
                attempts += 1;
                tracing::warn!(error = %e, attempts, "database not ready, retrying");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            Err(e) => return Err(anyhow::anyhow!("could not run migrations: {e}")),
        }
    }
}
