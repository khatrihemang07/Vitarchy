//! Why a read produced no result. None of these carry file content or passwords.

/// What can go wrong running the helper.
#[derive(Debug, thiserror::Error)]
pub enum ReaderError {
    /// The helper could not be started (missing program, bad permissions).
    #[error("could not start the statement reader: {0}")]
    Spawn(#[source] std::io::Error),
    /// Talking to the helper's pipes failed.
    #[error("could not talk to the statement reader: {0}")]
    Io(#[source] std::io::Error),
    /// The helper ran past its time limit and was killed.
    #[error("the statement reader timed out")]
    Timeout,
    /// The helper exited with a failure status (a crash).
    #[error("the statement reader crashed (exit {0:?})")]
    Crashed(Option<i32>),
    /// The helper's output was not a result this server understands.
    #[error("the statement reader's answer was unreadable")]
    Malformed,
}

impl ReaderError {
    /// Stable label for `runs.error_code` and logs.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Spawn(_) => "reader_unavailable",
            Self::Io(_) => "reader_io",
            Self::Timeout => "reader_timeout",
            Self::Crashed(_) => "reader_crashed",
            Self::Malformed => "reader_malformed",
        }
    }
}
