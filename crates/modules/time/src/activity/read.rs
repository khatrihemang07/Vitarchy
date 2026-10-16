//! Reads the Snapshot's `ZMANAGEDACTIVITY` table, read-only.

use rusqlite::{Connection, OpenFlags};

use super::{error::ActivityError, parse::RawRow, snapshot_path::SnapshotFile};

const QUERY: &str = "SELECT Z_PK, Z_ENT, Z_OPT, ZISIDLE, ZSYNCSTATUS, ZSTART, ZEND, \
                     ZFILENAME, ZTITLE, ZCLIENT, ZID FROM ZMANAGEDACTIVITY";

/// Every row of the Snapshot, read on a blocking thread.
///
/// # Errors
/// [`ActivityError::NoSnapshot`] when the file is absent; [`ActivityError::Unreadable`]
/// when it cannot be opened or lacks the expected table or columns.
pub async fn read_rows(file: &SnapshotFile) -> Result<Vec<RawRow>, ActivityError> {
    let path = file.path().to_path_buf();
    tokio::task::spawn_blocking(move || {
        if !path.is_file() {
            return Err(ActivityError::NoSnapshot);
        }
        read_blocking(&path).map_err(|error| {
            tracing::warn!(%error, "snapshot could not be read");
            ActivityError::Unreadable
        })
    })
    .await
    .map_err(|_| ActivityError::Unreadable)?
}

fn read_blocking(path: &std::path::Path) -> rusqlite::Result<Vec<RawRow>> {
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut statement = connection.prepare(QUERY)?;
    statement
        .query_map([], |row| {
            Ok(RawRow {
                pk: row.get(0)?,
                ent: row.get(1)?,
                opt: row.get(2)?,
                is_idle: row.get(3)?,
                sync_status: row.get(4)?,
                start: row.get(5)?,
                end: row.get(6)?,
                app: row.get(7)?,
                title: row.get(8)?,
                client: row.get(9)?,
                id: row.get(10)?,
            })
        })?
        .collect()
}
