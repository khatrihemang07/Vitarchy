//! Reading a Snapshot for any Source: the blocking-thread and error handling, once.

use super::{error::ActivityError, model::Parsed, snapshot_path::SnapshotFile, source::Source};

/// A Snapshot read and parsed: how many rows it held, and what became of them.
#[derive(Debug, Default, PartialEq)]
pub struct SnapshotRows {
    /// Rows found in the Snapshot.
    pub read: usize,
    /// The parsed Activity and the rows left out.
    pub parsed: Parsed,
}

/// Reads and parses `source`'s Snapshot on a blocking thread.
///
/// # Errors
/// [`ActivityError::NoSnapshot`] when the file is absent; [`ActivityError::Unreadable`]
/// when it cannot be opened or lacks the expected table or columns.
pub async fn read_snapshot(
    source: Source,
    file: &SnapshotFile,
) -> Result<SnapshotRows, ActivityError> {
    let path = file.path().to_path_buf();
    let reader = source.descriptor().reader;
    tokio::task::spawn_blocking(move || {
        if !path.is_file() {
            return Err(ActivityError::NoSnapshot);
        }
        reader(&path).map_err(|error| {
            tracing::warn!(source = source.id(), %error, "snapshot could not be read");
            ActivityError::Unreadable
        })
    })
    .await
    .map_err(|_| ActivityError::Unreadable)?
}
