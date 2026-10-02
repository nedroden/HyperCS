use std::{env, net::SocketAddr};

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub database_url: String,
}

impl Config {
    /// Load configuration from environment variables. Panics on invalid config at startup.
    pub fn from_env() -> Self {
        let bind_addr = env::var("BIND_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:8080".into())
            .parse()
            .expect("BIND_ADDR must be a valid socket address");
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        Self {
            bind_addr,
            database_url,
        }
    }
}
