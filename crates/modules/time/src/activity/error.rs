//! Why an Import could not start or finish.

/// A failed Import. Carries no row content, so it is safe to log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ActivityError {
    /// There is no Snapshot file: the helper has not run yet.
    #[error("no Snapshot yet")]
    NoSnapshot,
    /// The Snapshot exists but could not be read, or is not shaped as expected.
    #[error("the Snapshot could not be read")]
    Unreadable,
}

impl ActivityError {
    /// Stable label for `runs.error_code` and the problem's `code`.
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Self::NoSnapshot => "no_snapshot",
            Self::Unreadable => "snapshot_unreadable",
        }
    }
}
