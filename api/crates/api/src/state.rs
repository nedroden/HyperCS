use hypercs_domain::constants::{
    DUPLICATE_POST_WINDOW_SECS, MIN_POST_INTERVAL_SECS, NEW_ACCOUNT_AGE_MINS,
    NEW_ACCOUNT_POST_INTERVAL_SECS,
};
use sqlx::PgPool;

/// Flood-control and duplicate-post thresholds. Configurable (environment
/// variables in the server) so tests can switch them off with
/// [`FloodConfig::disabled`].
#[derive(Debug, Clone, Copy)]
pub struct FloodConfig {
    /// Minimum seconds between two posts by the same account.
    pub min_interval_secs: i64,
    /// Stricter minimum for accounts younger than `new_account_age_mins`.
    pub new_account_interval_secs: i64,
    pub new_account_age_mins: i64,
    /// An identical post by the same user within this window is rejected.
    pub duplicate_window_secs: i64,
}

impl Default for FloodConfig {
    fn default() -> Self {
        Self {
            min_interval_secs: MIN_POST_INTERVAL_SECS,
            new_account_interval_secs: NEW_ACCOUNT_POST_INTERVAL_SECS,
            new_account_age_mins: NEW_ACCOUNT_AGE_MINS,
            duplicate_window_secs: DUPLICATE_POST_WINDOW_SECS,
        }
    }
}

impl FloodConfig {
    /// No flood control and no duplicate guard (used by most tests).
    pub fn disabled() -> Self {
        Self {
            min_interval_secs: 0,
            new_account_interval_secs: 0,
            new_account_age_mins: 0,
            duplicate_window_secs: 0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ApiConfig {
    /// Sets the `Secure` flag on session cookies (and enables HSTS); disable
    /// only for local http development.
    pub cookie_secure: bool,
    /// Explicit allow-list of origins, empty by default. Used both for CORS
    /// and as additional trusted origins for the CSRF `Origin` check (e.g.
    /// the Vite dev server when it proxies with a rewritten `Host`).
    pub allowed_origins: Vec<String>,
    /// Serves Swagger UI at `/api/v1/docs` when true.
    pub docs_enabled: bool,
    pub flood: FloodConfig,
}

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: ApiConfig,
}
