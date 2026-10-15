use tracing_subscriber::{EnvFilter, fmt};

/// Installs the global tracing subscriber; level comes from `RUST_LOG` (default `info`).
/// Logs go to stderr so stdout stays free for machine-readable output.
pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
}
