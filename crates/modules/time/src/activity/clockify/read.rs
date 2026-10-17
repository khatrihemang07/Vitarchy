//! Reads the Clockify Snapshot's `ZCDAUTOTRACKERITEM` table, read-only.

use std::path::Path;

use rusqlite::{Connection, OpenFlags};

use super::parse::{RawRow, parse_all};
use crate::activity::read_snapshot::SnapshotRows;

// The icon blob (`ZICONDATA`, several KB per row) is left out on purpose.
const QUERY: &str = "SELECT Z_PK, Z_ENT, Z_OPT, ZITEMNO, ZTIMEENTRYADDED, ZIDLETIME, \
                     ZTIMESTARTED, ZTIMEENDED, ZCOLOR, ZICONID, ZID, ZITEMDESCRIPTION, \
                     ZITEMURL, ZNAME FROM ZCDAUTOTRACKERITEM";

/// Reads and parses a Clockify Snapshot.
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
                item_no: row.get(3)?,
                time_entry_added: row.get(4)?,
                idle_time: row.get(5)?,
                started: row.get(6)?,
                ended: row.get(7)?,
                color: row.get(8)?,
                icon_id: row.get(9)?,
                id: row.get(10)?,
                description: row.get(11)?,
                url: row.get(12)?,
                name: row.get(13)?,
            })
        })?
        .collect()
}
