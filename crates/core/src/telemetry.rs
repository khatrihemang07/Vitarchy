//! Process-wide logging setup: output format and level filter.

use std::str::FromStr;

use tracing_subscriber::{EnvFilter, fmt};

/// Level used when `RUST_LOG` is unset: `info` for everything, with the noisy
/// dependencies quieted to `warn`. It names no Vitarchy crate, so a new Module
/// logs at `info` without touching this file.
pub const DEFAULT_FILTER: &str = "info,sqlx=warn,hyper=warn,hyper_util=warn,h2=warn,tower=warn";

/// Name of the environment variable that selects the [`LogFormat`].
pub const LOG_FORMAT_VAR: &str = "LOG_FORMAT";

/// How log lines are rendered.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LogFormat {
    /// Human-readable lines for the terminal during development.
    #[default]
    Pretty,
    /// One JSON object per line, for tools that parse logs.
    Json,
}

/// `LOG_FORMAT` held something other than `pretty` or `json`.
#[derive(Debug, PartialEq, Eq)]
pub struct UnknownLogFormat(pub String);

impl FromStr for LogFormat {
    type Err = UnknownLogFormat;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "" | "pretty" => Ok(Self::Pretty),
            "json" => Ok(Self::Json),
            other => Err(UnknownLogFormat(other.to_owned())),
        }
    }
}

impl LogFormat {
    /// Reads the format from the raw `LOG_FORMAT` value; unset means the default.
    ///
    /// # Errors
    /// [`UnknownLogFormat`] when the value is set but not recognised.
    pub fn from_var(raw: Option<&str>) -> Result<Self, UnknownLogFormat> {
        raw.map_or(Ok(Self::default()), str::parse)
    }
}

/// Installs the global tracing subscriber. The level comes from `RUST_LOG`
/// (default [`DEFAULT_FILTER`]) and the format from `LOG_FORMAT`.
/// Logs go to stderr so stdout stays free for machine-readable output.
pub fn init() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));
    let raw = std::env::var(LOG_FORMAT_VAR).ok();
    let format = LogFormat::from_var(raw.as_deref());

    let builder = fmt().with_env_filter(filter).with_writer(std::io::stderr);
    match format {
        Ok(LogFormat::Json) => builder.json().init(),
        Ok(LogFormat::Pretty) => builder.init(),
        Err(UnknownLogFormat(bad)) => {
            builder.init();
            tracing::warn!(value = %bad, "unknown {LOG_FORMAT_VAR}, using pretty (expected pretty or json)");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unset_or_empty_log_format_is_pretty() {
        assert_eq!(LogFormat::from_var(None), Ok(LogFormat::Pretty));
        assert_eq!(LogFormat::from_var(Some("")), Ok(LogFormat::Pretty));
    }

    #[test]
    fn log_format_is_case_and_space_insensitive() {
        assert_eq!(LogFormat::from_var(Some(" JSON ")), Ok(LogFormat::Json));
        assert_eq!(LogFormat::from_var(Some("Pretty")), Ok(LogFormat::Pretty));
    }

    #[test]
    fn unknown_log_format_is_reported_with_its_value() {
        assert_eq!(
            LogFormat::from_var(Some("xml")),
            Err(UnknownLogFormat("xml".to_owned()))
        );
    }

    #[test]
    fn default_filter_is_a_valid_env_filter() {
        assert!(EnvFilter::try_new(DEFAULT_FILTER).is_ok());
    }

    #[test]
    fn default_filter_names_no_vitarchy_crate() {
        assert!(!DEFAULT_FILTER.contains("vitarchy"));
    }
}
