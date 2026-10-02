use axum::Router;
use sqlx::PgPool;

mod health;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1", Router::new().merge(health::routes()))
        .with_state(state)
}
