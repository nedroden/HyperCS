use std::{env, net::SocketAddr};

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub database_url: String,
    pub cookie_secure: bool,
    pub allowed_origins: Vec<String>,
    pub docs_enabled: bool,
    pub run_migrations: bool,
    pub admin_username: Option<String>,
    pub admin_email: Option<String>,
    pub admin_password: Option<String>,
}

fn env_bool(name: &str, default: bool) -> bool {
    env::var(name)
        .ok()
        .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(default)
}

impl Config {
    /// Loads configuration from environment variables. Panics on invalid
    /// config at startup (fine outside request paths).
    pub fn from_env() -> Self {
        let bind_addr = env::var("BIND_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:8080".into())
            .parse()
            .expect("BIND_ADDR must be a valid socket address");
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let cookie_secure = env_bool("COOKIE_SECURE", false);
        let allowed_origins = env::var("ALLOWED_ORIGINS")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
        let docs_enabled = env_bool("API_DOCS_ENABLED", cfg!(debug_assertions));
        let run_migrations = env_bool("RUN_MIGRATIONS_ON_START", true);
        Self {
            bind_addr,
            database_url,
            cookie_secure,
            allowed_origins,
            docs_enabled,
            run_migrations,
            admin_username: env::var("ADMIN_USERNAME").ok(),
            admin_email: env::var("ADMIN_EMAIL").ok(),
            admin_password: env::var("ADMIN_PASSWORD").ok(),
        }
    }

    pub fn api_config(&self) -> hypercs_api::state::ApiConfig {
        hypercs_api::state::ApiConfig {
            cookie_secure: self.cookie_secure,
            allowed_origins: self.allowed_origins.clone(),
            docs_enabled: self.docs_enabled,
        }
    }
}
