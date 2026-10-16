//! The structured `key=value` log lines the helper writes to stderr.
//!
//! Only outcomes, sizes, timings and paths are logged, never database content
//! such as window titles.

use std::time::Duration;

use crate::snapshot::{Snapshot, SnapshotError};

/// The line for a finished Snapshot.
#[must_use]
pub fn success_line(snapshot: &Snapshot, elapsed: Duration) -> String {
    format!(
        "level=info msg=\"snapshot taken\" outcome=ok bytes={} duration_ms={} dest={:?}",
        snapshot.bytes,
        elapsed.as_millis(),
        snapshot.path.display().to_string()
    )
}

/// The line for a failed Snapshot.
#[must_use]
pub fn failure_line(error: &SnapshotError, elapsed: Duration) -> String {
    format!(
        "level=error msg=\"snapshot failed\" outcome=error kind={} duration_ms={} error={:?}",
        error.kind(),
        elapsed.as_millis(),
        error.to_string()
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn success_line_has_outcome_bytes_and_duration() {
        let snapshot = Snapshot {
            path: PathBuf::from("/d/latest.sqlite"),
            bytes: 4096,
        };
        let line = success_line(&snapshot, Duration::from_millis(12));
        assert!(line.contains("outcome=ok"));
        assert!(line.contains("bytes=4096"));
        assert!(line.contains("duration_ms=12"));
    }

    #[test]
    fn failure_line_has_outcome_and_kind() {
        let error = SnapshotError::SourceMissing(PathBuf::from("/nope"));
        let line = failure_line(&error, Duration::from_millis(1));
        assert!(line.contains("outcome=error"));
        assert!(line.contains("kind=source_missing"));
    }
}
