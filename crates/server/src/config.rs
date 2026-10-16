//! Server configuration read from environment variables.

use std::{env, net::SocketAddr, path::PathBuf};

use thiserror::Error;

const DEFAULT_BIND_ADDR: &str = "127.0.0.1:3000";

/// Why the environment did not make a valid [`Config`].
#[derive(Debug, Error)]
pub enum ConfigError {
    /// A required variable is unset.
    #[error("{0} must be set (see .env.example)")]
    Missing(&'static str),
    /// `BIND_ADDR` is not `host:port`.
    #[error("BIND_ADDR is not a valid socket address: {0}")]
    BindAddr(#[from] std::net::AddrParseError),
    /// `BIND_ADDR` would listen beyond this machine.
    #[error("BIND_ADDR must be a loopback address (Vitarchy is local-only), got {0}")]
    NotLoopback(SocketAddr),
}

/// Unvalidated configuration values, as read from the environment.
#[derive(Debug, Default)]
pub struct RawConfig {
    /// `DATABASE_URL`.
    pub database_url: Option<String>,
    /// `BIND_ADDR`.
    pub bind_addr: Option<String>,
    /// `WEB_DIST`.
    pub web_dist: Option<String>,
}

/// Validated server configuration.
pub struct Config {
    /// Postgres connection string; contains a password, so log it only redacted.
    pub database_url: String,
    /// Loopback address the server listens on.
    pub bind_addr: SocketAddr,
    /// Built web app to serve beside the API (`make start`). `None` in dev,
    /// where Vite serves the UI.
    pub web_dist: Option<PathBuf>,
}

impl Config {
    /// Reads configuration from environment variables.
    ///
    /// # Errors
    /// See [`Config::parse`].
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::parse(RawConfig {
            database_url: env::var("DATABASE_URL").ok(),
            bind_addr: env::var("BIND_ADDR").ok(),
            web_dist: env::var("WEB_DIST").ok(),
        })
    }

    /// Builds configuration from raw values.
    ///
    /// # Errors
    /// Fails when `database_url` is missing, or `bind_addr` is malformed or not a
    /// loopback address: the server never listens beyond this machine.
    pub fn parse(raw: RawConfig) -> Result<Self, ConfigError> {
        let RawConfig {
            database_url,
            bind_addr,
            web_dist,
        } = raw;
        let bind_addr: SocketAddr = bind_addr
            .unwrap_or_else(|| DEFAULT_BIND_ADDR.to_owned())
            .parse()?;
        if !bind_addr.ip().is_loopback() {
            return Err(ConfigError::NotLoopback(bind_addr));
        }
        Ok(Self {
            database_url: database_url.ok_or(ConfigError::Missing("DATABASE_URL"))?,
            bind_addr,
            // An empty value (e.g. `WEB_DIST=` in .env) means "not set".
            web_dist: web_dist.filter(|d| !d.is_empty()).map(PathBuf::from),
        })
    }
}
