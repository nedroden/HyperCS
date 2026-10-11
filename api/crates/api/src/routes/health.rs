use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;
use utoipa::ToSchema;

use crate::state::AppState;

#[derive(Serialize, ToSchema)]
pub struct Health {
    /// Always `ok` while the process is alive.
    #[schema(example = "ok")]
    status: &'static str,
    version: &'static str,
    /// `up` or `down`.
    #[schema(example = "up")]
    database: &'static str,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health))
}

/// Liveness probe.
#[utoipa::path(get, path = "/api/v1/health", operation_id = "health", tag = "misc",
    responses((status = 200, description = "The service is alive.", body = Health)))]
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
