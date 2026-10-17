//! Reads the Snapshot's `ZMANAGEDACTIVITY` table, read-only.

use std::path::Path;

use rusqlite::{Connection, OpenFlags};

use super::parse::{RawRow, parse_all};
use crate::activity::read_snapshot::SnapshotRows;

const QUERY: &str = "SELECT Z_PK, Z_ENT, Z_OPT, ZISIDLE, ZSYNCSTATUS, ZSTART, ZEND, \
                     ZFILENAME, ZTITLE, ZCLIENT, ZID FROM ZMANAGEDACTIVITY";

/// Reads and parses a Toggl Snapshot.
///
/// # Errors
/// When the file cannot be opened or lacks the expected table or columns.
pub fn read_snapshot_rows(path: &Path) -> rusqlite::Result<SnapshotRows> {
    let rows = read_rows(path)?;
    Ok(SnapshotRows {
        read: rows.len(),
        parsed: parse_all(&rows),
    })
}

fn read_rows(path: &Path) -> rusqlite::Result<Vec<RawRow>> {
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
