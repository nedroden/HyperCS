use std::{env, net::SocketAddr};

use hypercs_api::state::{ApiConfig, FloodConfig};

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub database_url: String,
    pub cookie_secure: bool,
    pub allowed_origins: Vec<String>,
    pub docs_enabled: bool,
    pub run_migrations: bool,
    pub json_logs: bool,
    pub flood: FloodConfig,
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

fn env_i64(name: &str, default: i64) -> i64 {
    match env::var(name) {
        Ok(v) if !v.trim().is_empty() => v
            .trim()
            .parse()
            .unwrap_or_else(|_| panic!("{name} must be an integer")),
        _ => default,
    }
}

fn env_nonempty(name: &str) -> Option<String> {
    env::var(name).ok().filter(|v| !v.trim().is_empty())
}

/// An allowed origin is `scheme://host[:port]` without path or trailing slash.
pub fn is_valid_origin(origin: &str) -> bool {
    let Some(rest) = origin
        .strip_prefix("https://")
        .or_else(|| origin.strip_prefix("http://"))
    else {
        return false;
    };
    !rest.is_empty() && !rest.contains(['/', '?', '#', '*', ' '])
}

pub fn parse_allowed_origins(value: &str) -> Result<Vec<String>, String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|o| {
            if is_valid_origin(o) {
                Ok(o.to_string())
            } else {
                Err(format!(
                    "ALLOWED_ORIGINS entry {o:?} must look like https://example.com \
                     (no path, no trailing slash, no wildcard)"
                ))
            }
        })
        .collect()
}

impl Config {
    /// Loads configuration from environment variables. Panics on invalid
    /// config at startup (fine outside request paths).
    pub fn from_env() -> Self {
        let defaults = FloodConfig::default();
        Self {
            bind_addr: env::var("BIND_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".into())
                .parse()
                .expect("BIND_ADDR must be a valid socket address"),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            cookie_secure: env_bool("COOKIE_SECURE", false),
            allowed_origins: parse_allowed_origins(
                &env::var("ALLOWED_ORIGINS").unwrap_or_default(),
            )
            .unwrap_or_else(|e| panic!("{e}")),
            docs_enabled: env_bool("API_DOCS_ENABLED", cfg!(debug_assertions)),
            run_migrations: env_bool("RUN_MIGRATIONS_ON_START", cfg!(debug_assertions)),
            json_logs: env::var("LOG_FORMAT").is_ok_and(|v| v.eq_ignore_ascii_case("json")),
            flood: FloodConfig {
                min_interval_secs: env_i64("POST_MIN_INTERVAL_SECS", defaults.min_interval_secs),
                new_account_interval_secs: env_i64(
                    "POST_NEW_ACCOUNT_INTERVAL_SECS",
                    defaults.new_account_interval_secs,
                ),
                new_account_age_mins: env_i64(
                    "POST_NEW_ACCOUNT_AGE_MINS",
                    defaults.new_account_age_mins,
                ),
                duplicate_window_secs: env_i64(
                    "POST_DUPLICATE_WINDOW_SECS",
                    defaults.duplicate_window_secs,
                ),
            },
            admin_username: env_nonempty("ADMIN_USERNAME"),
            admin_email: env_nonempty("ADMIN_EMAIL"),
            admin_password: env_nonempty("ADMIN_PASSWORD"),
        }
    }

    pub fn api_config(&self) -> ApiConfig {
        ApiConfig {
            cookie_secure: self.cookie_secure,
            allowed_origins: self.allowed_origins.clone(),
            docs_enabled: self.docs_enabled,
            flood: self.flood,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origins() {
        assert_eq!(
            parse_allowed_origins("http://localhost:5173, https://example.com").unwrap(),
            vec!["http://localhost:5173", "https://example.com"]
        );
        assert!(parse_allowed_origins("").unwrap().is_empty());
        assert!(parse_allowed_origins("https://example.com/").is_err());
        assert!(parse_allowed_origins("*").is_err());
        assert!(parse_allowed_origins("example.com").is_err());
    }
}
