//! A fixture Clockify Snapshot: a `SQLite` file with the Clockify Desktop `ZCDAUTOTRACKERITEM` shape.

use std::path::PathBuf;

use rusqlite::{Connection, params};
use tempfile::TempDir;
use vitarchy_time::SnapshotFile;

/// A row to put in the fixture. Times are Core Data seconds.
pub struct Item {
    pub id: String,
    pub started: i64,
    pub ended: i64,
    pub name: &'static str,
    pub url: &'static str,
    pub idle: f64,
}

impl Item {
    pub fn new(id: &str, started: i64, ended: i64) -> Self {
        Self {
            id: id.to_owned(),
            started,
            ended,
            name: "Code",
            url: "",
            idle: 0.0,
        }
    }
}

/// A Clockify Snapshot in a temporary folder.
pub struct ClockifyFixture {
    dir: TempDir,
}

impl ClockifyFixture {
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

    /// Replaces the Snapshot with one holding `items`.
    pub fn write(&self, items: &[Item]) {
        let _ = std::fs::remove_file(self.path());
        let db = Connection::open(self.path()).unwrap();
        db.execute_batch(
            "CREATE TABLE ZCDAUTOTRACKERITEM ( Z_PK INTEGER PRIMARY KEY, Z_ENT INTEGER,
               Z_OPT INTEGER, ZITEMNO INTEGER, ZTIMEENTRYADDED INTEGER, ZIDLETIME FLOAT,
               ZTIMEENDED TIMESTAMP, ZTIMESTARTED TIMESTAMP, ZCOLOR VARCHAR, ZICONID VARCHAR,
               ZID VARCHAR, ZITEMDESCRIPTION VARCHAR, ZITEMURL VARCHAR, ZNAME VARCHAR,
               ZICONDATA BLOB );",
        )
        .unwrap();
        for (pk, item) in (1_i64..).zip(items) {
            Self::insert(&db, pk, item);
        }
    }

    /// Adds a row that ends before it starts.
    pub fn add_backwards_item(&self) {
        let db = Connection::open(self.path()).unwrap();
        Self::insert(&db, 901, &Item::new("backwards", 10, 5));
    }

    fn insert(db: &Connection, pk: i64, item: &Item) {
        db.execute(
            "INSERT INTO ZCDAUTOTRACKERITEM VALUES
               (?1, 1, 1, ?1, 0, ?2, ?3, ?4, '#263238', ?5, ?6, 'a title', ?7, ?5, x'00ff')",
            params![
                pk,
                item.idle,
                item.ended,
                item.started,
                item.name,
                &item.id,
                item.url
            ],
        )
        .unwrap();
    }
}
