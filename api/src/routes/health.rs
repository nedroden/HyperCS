use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;

use super::AppState;

#[derive(Serialize)]
struct Health {
    status: &'static str,
    version: &'static str,
    database: &'static str,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health))
}

async fn health(State(state): State<AppState>) -> Json<Health> {
    let database = match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => "up",
        Err(_) => "down",
    };
    Json(Health {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        database,
    })
}
