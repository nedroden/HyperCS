use axum::{Json, Router, extract::State, routing::get};

use crate::{error::ApiResult, services::stats_service, state::AppState};

pub fn routes() -> Router<AppState> {
    Router::new().route("/stats", get(get_stats))
}

#[utoipa::path(get, path = "/api/v1/stats", responses((status = 200, body = hypercs_db::stats::Stats)), tag = "misc")]
async fn get_stats(State(state): State<AppState>) -> ApiResult<Json<hypercs_db::stats::Stats>> {
    Ok(Json(stats_service::get_stats(&state.db).await?))
}
