use clap::{Parser, Subcommand};
use hypercs_api::{routes, state::AppState};
use hypercs_server::{config::Config, connect, housekeeping, recount, run_migrations, seed};
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
    /// Recomputes denormalized counters from the source tables and exits.
    Recount,
    /// Prints the OpenAPI document to stdout and exits.
    Openapi,
}

fn init_tracing(json: bool) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
    let builder = tracing_subscriber::fmt().with_env_filter(filter);
    if json {
        builder.json().init();
    } else {
        builder.init();
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Command::Serve);

    // `openapi` must work without any environment (client generation and
    // drift checks), so it runs before the configuration is read.
    if matches!(command, Command::Openapi) {
        let doc = <hypercs_api::openapi::ApiDoc as utoipa::OpenApi>::openapi();
        println!("{}", doc.to_pretty_json()?);
        return Ok(());
    }

    let config = Config::from_env();
    init_tracing(config.json_logs);

    match command {
        Command::Serve => serve(config).await,
        Command::Migrate => {
            let db = connect(&config)?;
            run_migrations(&db).await
        }
        Command::Recount => {
            let db = connect(&config)?;
            recount::run(&db).await.map(|_| ())
        }
        Command::Openapi => Ok(()),
    }
}

async fn serve(config: Config) -> anyhow::Result<()> {
    let db = connect(&config)?;

    if config.run_migrations {
        run_migrations(&db).await?;
    }
    if let Err(error) = seed::bootstrap_admin(&db, &config).await {
        tracing::error!(%error, "failed to bootstrap the admin account");
    }
    if let Err(error) = seed::seed_sample_structure(&db).await {
        tracing::error!(%error, "failed to seed the sample forum structure");
    }

    housekeeping::spawn_session_purge(db.clone());

    let app = routes::router(AppState {
        db,
        config: config.api_config(),
    });
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    tracing::info!("listening on {}", config.bind_addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
