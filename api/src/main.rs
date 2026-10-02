use hypercs_api::{
    config::Config,
    routes::{self, AppState},
};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = Config::from_env();

    // Lazy pool: the API starts even if the database is still booting; /health reports its state.
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect_lazy(&config.database_url)
        .expect("DATABASE_URL must be a valid connection string");

    // Retry migrations until the database is reachable (e.g. during `docker compose up`).
    let mut attempts = 0;
    loop {
        match sqlx::migrate!("./migrations").run(&db).await {
            Ok(()) => break,
            Err(e) if attempts < 30 => {
                attempts += 1;
                tracing::warn!(error = %e, attempts, "database not ready, retrying");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            Err(e) => panic!("could not run migrations: {e}"),
        }
    }

    let app = routes::router(AppState { db });
    let listener = tokio::net::TcpListener::bind(config.bind_addr)
        .await
        .expect("could not bind");
    tracing::info!("listening on {}", config.bind_addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .expect("server error");
}
