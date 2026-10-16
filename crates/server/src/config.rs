//! Server configuration read from environment variables.

use std::{env, net::SocketAddr};

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

/// Validated server configuration.
pub struct Config {
    /// Postgres connection string; contains a password, so log it only redacted.
    pub database_url: String,
    /// Loopback address the server listens on.
    pub bind_addr: SocketAddr,
}

impl Config {
    /// Reads configuration from environment variables.
    ///
    /// # Errors
    /// See [`Config::parse`].
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::parse(env::var("DATABASE_URL").ok(), env::var("BIND_ADDR").ok())
    }

    /// Builds configuration from raw values.
    ///
    /// # Errors
    /// Fails when `database_url` is missing, or `bind_addr` is malformed or not a
    /// loopback address: the server never listens beyond this machine.
    pub fn parse(
        database_url: Option<String>,
        bind_addr: Option<String>,
    ) -> Result<Self, ConfigError> {
        let bind_addr: SocketAddr = bind_addr
            .unwrap_or_else(|| DEFAULT_BIND_ADDR.to_owned())
            .parse()?;
        if !bind_addr.ip().is_loopback() {
            return Err(ConfigError::NotLoopback(bind_addr));
        }
        Ok(Self {
            database_url: database_url.ok_or(ConfigError::Missing("DATABASE_URL"))?,
            bind_addr,
        })
    }
}
