//! A fixture Snapshot: a `SQLite` file with the Toggl Track `ZMANAGEDACTIVITY` shape and synthetic rows.

use std::path::PathBuf;

use rusqlite::{Connection, params};
use tempfile::TempDir;
use vitarchy_time::SnapshotFile;

/// A row to put in the fixture. Times are Core Data seconds.
pub struct Row {
    pub id: u8,
    pub start: f64,
    pub end: f64,
    pub app: &'static str,
    pub title: &'static str,
    pub idle: i64,
}

impl Row {
    pub fn new(id: u8, start: f64, end: f64) -> Self {
        Self {
            id,
            start,
            end,
            app: "AppOne",
            title: "",
            idle: 0,
        }
    }
}

/// 16 bytes whose last byte is `last`.
pub fn blob(last: u8) -> Vec<u8> {
    let mut bytes = vec![0xAB; 15];
    bytes.push(last);
    bytes
}

/// The UUID text of `blob(last)`.
pub fn uuid(last: u8) -> String {
    format!("abababab-abab-abab-abab-ababababab{last:02x}")
}

/// A Snapshot in a temporary folder.
pub struct Fixture {
    dir: TempDir,
}

impl Fixture {
    pub fn new() -> Self {
        Self {
            dir: TempDir::new().unwrap(),
        }
    }

    pub fn path(&self) -> PathBuf {
        self.dir.path().join("latest.sqlite")
    }

    pub fn file(&self) -> SnapshotFile {
        SnapshotFile::at(self.path())
    }

    /// Replaces the Snapshot with one holding `rows`.
    pub fn write(&self, rows: &[Row]) {
        let _ = std::fs::remove_file(self.path());
        let db = Connection::open(self.path()).unwrap();
        db.execute_batch(
            "CREATE TABLE ZMANAGEDACTIVITY (Z_PK INTEGER PRIMARY KEY, Z_ENT INTEGER, Z_OPT INTEGER,
               ZISIDLE INTEGER, ZSYNCSTATUS INTEGER, ZSTART TIMESTAMP, ZEND TIMESTAMP,
               ZFILENAME VARCHAR, ZTITLE VARCHAR, ZCLIENT BLOB, ZID BLOB);",
        )
        .unwrap();
        for (pk, row) in (1_i64..).zip(rows) {
            Self::insert(&db, pk, row, &blob(row.id));
        }
    }

    /// Adds a row whose id is not 16 bytes.
    pub fn add_row_with_bad_id(&self) {
        let db = Connection::open(self.path()).unwrap();
        Self::insert(&db, 900, &Row::new(0, 1.0, 2.0), &[1, 2, 3]);
    }

    /// Adds a row that ends before it starts.
    pub fn add_backwards_row(&self) {
        let db = Connection::open(self.path()).unwrap();
        Self::insert(&db, 901, &Row::new(99, 10.0, 5.0), &blob(99));
    }

    /// Writes a file that is not a `SQLite` database.
    pub fn write_garbage(&self) {
        std::fs::write(self.path(), b"this is not sqlite at all, just text").unwrap();
    }

    fn insert(db: &Connection, pk: i64, row: &Row, id: &[u8]) {
        db.execute(
            "INSERT INTO ZMANAGEDACTIVITY VALUES (?1, 1, 1, ?2, 0, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                pk,
                row.idle,
                row.start,
                row.end,
                row.app,
                row.title,
                blob(0),
                id
            ],
        )
        .unwrap();
    }
}
