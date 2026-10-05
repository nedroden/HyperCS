use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// Sets the `Secure` flag on session cookies; disable only for local http dev.
    pub cookie_secure: bool,
    /// Explicit allow-list for CORS/Origin checks; empty by default (same-origin deployment).
    pub allowed_origins: Vec<String>,
    /// Serves Swagger UI at `/api/v1/docs` when true.
    pub docs_enabled: bool,
}

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: ApiConfig,
}
