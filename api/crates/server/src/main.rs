mod config;
mod housekeeping;
mod recount;
mod seed;

use clap::{Parser, Subcommand};
use config::Config;
use hypercs_api::{routes, services::auth_service, state::AppState};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "hypercs-server")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Starts the HTTP server (default when no subcommand is given).
    Serve,
    /// Runs pending migrations and exits.
    Migrate,
    /// Recomputes denormalized counters from source tables and exits.
    Recount,
    /// Prints the OpenAPI document to stdout and exits.
    Openapi,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let cli = Cli::parse();
    match cli.command.unwrap_or(Command::Serve) {
        Command::Serve => serve().await,
        Command::Migrate => migrate().await,
        Command::Recount => {
            let config = Config::from_env();
            let db = connect(&config).await?;
            recount::run(&db).await
        }
        Command::Openapi => {
            let doc = <hypercs_api::openapi::ApiDoc as utoipa::OpenApi>::openapi();
            println!("{}", doc.to_pretty_json()?);
            Ok(())
        }
    }
}

async fn connect(config: &Config) -> anyhow::Result<sqlx::PgPool> {
    Ok(PgPoolOptions::new()
        .max_connections(10)
        .connect_lazy(&config.database_url)?)
}

async fn migrate() -> anyhow::Result<()> {
    let config = Config::from_env();
    let db = connect(&config).await?;
    run_migrations_with_retry(&db).await;
    Ok(())
}

async fn run_migrations_with_retry(db: &sqlx::PgPool) {
    let mut attempts = 0;
    loop {
        match sqlx::migrate!("../db/migrations").run(db).await {
            Ok(()) => break,
            Err(e) if attempts < 30 => {
                attempts += 1;
                tracing::warn!(error = %e, attempts, "database not ready, retrying");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            Err(e) => panic!("could not run migrations: {e}"),
        }
    }
}

async fn serve() -> anyhow::Result<()> {
    let config = Config::from_env();
    let db = connect(&config).await?;

    if config.run_migrations {
        run_migrations_with_retry(&db).await;
    }

    if let (Some(username), Some(email), Some(password)) = (
        &config.admin_username,
        &config.admin_email,
        &config.admin_password,
    ) {
        if let Err(error) = auth_service::bootstrap_admin(&db, username, email, password).await {
            tracing::error!(?error, "failed to bootstrap admin account");
        }
    } else {
        tracing::warn!(
            "ADMIN_USERNAME/ADMIN_EMAIL/ADMIN_PASSWORD not fully set; skipping admin bootstrap"
        );
    }

    if let Err(error) = seed::seed_sample_structure(&db).await {
        tracing::error!(?error, "failed to seed sample forum structure");
    }

    housekeeping::spawn_session_purge(db.clone());

    let state = AppState {
        db,
        config: config.api_config(),
    };
    let app = routes::router(state);
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    tracing::info!("listening on {}", config.bind_addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
