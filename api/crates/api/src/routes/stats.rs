use axum::{Json, Router, extract::State, routing::get};

use crate::{dto::Stats, error::ApiResult, services::stats_service, state::AppState};

pub fn routes() -> Router<AppState> {
    Router::new().route("/stats", get(get_stats))
}

/// Board-index statistics: totals, newest member, latest post, users online.
#[utoipa::path(get, path = "/api/v1/stats", operation_id = "get_stats", tag = "misc",
    responses((status = 200, body = Stats)))]
async fn get_stats(State(state): State<AppState>) -> ApiResult<Json<Stats>> {
    Ok(Json(stats_service::get_stats(&state.db).await?))
}
