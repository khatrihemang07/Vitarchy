//! Copies a source database with `SQLite`'s online backup API.
//!
//! The backup API reads through `SQLite` itself, so rows still sitting in the
//! write-ahead log are captured, which a plain file copy would miss.

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::backup::Backup;
use rusqlite::{Connection, OpenFlags};

use crate::paths::Paths;

/// Pages copied per backup step, and the pause when the source is busy.
const PAGES_PER_STEP: i32 = 256;
const BUSY_PAUSE: Duration = Duration::from_millis(50);

/// A finished Snapshot on disk.
#[derive(Debug)]
pub struct Snapshot {
    /// The Snapshot file. Its modification time is the Snapshot time.
    pub path: PathBuf,
    /// Its size in bytes.
    pub bytes: u64,
}

/// Why a Snapshot could not be taken.
#[derive(Debug, thiserror::Error)]
pub enum SnapshotError {
    /// The source database does not exist (the tracking app is not installed, or the path is wrong).
    #[error("source database not found at {}", .0.display())]
    SourceMissing(PathBuf),
    /// `SQLite` failed while opening or copying. Usually missing Full Disk Access.
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    /// A file operation on the destination failed.
    #[error("{action}: {source}")]
    Io {
        /// What was being done.
        action: &'static str,
        /// The underlying error.
        source: std::io::Error,
    },
}

impl SnapshotError {
    /// A short stable label for logs.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::SourceMissing(_) => "source_missing",
            Self::Sqlite(_) => "sqlite",
            Self::Io { .. } => "io",
        }
    }
}

fn io(action: &'static str) -> impl FnOnce(std::io::Error) -> SnapshotError {
    move |source| SnapshotError::Io { action, source }
}

/// Takes a Snapshot: backup into the temp file, flush it, then rename it over
/// the previous Snapshot so a reader never sees a half-written file.
///
/// # Errors
/// Returns a [`SnapshotError`] if the source is missing or unreadable, or the
/// destination cannot be written. A failed run leaves the previous Snapshot intact.
pub fn take(paths: &Paths) -> Result<Snapshot, SnapshotError> {
    if !paths.source.is_file() {
        return Err(SnapshotError::SourceMissing(paths.source.clone()));
    }
    fs::create_dir_all(&paths.dest_dir).map_err(io("create destination folder"))?;

    let temp = paths.temp();
    // A leftover from a crashed run would be merged into by the backup.
    remove_if_present(&temp)?;
    let result = copy_and_publish(&paths.source, &temp, &paths.snapshot());
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

fn copy_and_publish(
    source: &Path,
    temp: &Path,
    snapshot: &Path,
) -> Result<Snapshot, SnapshotError> {
    backup(source, temp)?;
    File::open(temp)
        .and_then(|file| file.sync_all())
        .map_err(io("flush snapshot"))?;
    fs::rename(temp, snapshot).map_err(io("replace snapshot"))?;
    let bytes = fs::metadata(snapshot)
        .map_err(io("read snapshot size"))?
        .len();
    Ok(Snapshot {
        path: snapshot.to_path_buf(),
        bytes,
    })
}

fn backup(source: &Path, temp: &Path) -> Result<(), SnapshotError> {
    let from = Connection::open_with_flags(source, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut to = Connection::open(temp)?;
    Backup::new(&from, &mut to)?.run_to_completion(PAGES_PER_STEP, BUSY_PAUSE, None)?;
    // The copy inherits the source's WAL mode. Rolling it back to a plain
    // journal leaves one self-contained file that readers can open read-only.
    to.pragma_update(None, "journal_mode", "DELETE")?;
    to.close().map_err(|(_, error)| error)?;
    Ok(())
}

fn remove_if_present(path: &Path) -> Result<(), SnapshotError> {
    match fs::remove_file(path) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            Err(io("remove stale temp file")(error))
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn paths_in(dir: &TempDir) -> Paths {
        Paths {
            source: dir.path().join("source.sqlite"),
            dest_dir: dir.path().join("out"),
        }
    }

    /// A WAL-mode database whose latest rows are still only in the WAL. The
    /// returned connection must stay open: closing it would checkpoint them.
    fn source_with_uncheckpointed_rows(path: &Path, rows: i64) -> Connection {
        let conn = Connection::open(path).unwrap();
        conn.pragma_update(None, "journal_mode", "WAL").unwrap();
        conn.pragma_update(None, "wal_autocheckpoint", 0).unwrap();
        conn.execute(
            "CREATE TABLE activity (id INTEGER PRIMARY KEY, app TEXT)",
            [],
        )
        .unwrap();
        for id in 0..rows {
            conn.execute(
                "INSERT INTO activity (app) VALUES (?1)",
                [format!("app{id}")],
            )
            .unwrap();
        }
        conn
    }

    fn count_rows(path: &Path) -> i64 {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        conn.query_row("SELECT count(*) FROM activity", [], |row| row.get(0))
            .unwrap()
    }

    #[test]
    fn captures_rows_that_are_only_in_the_wal() {
        let dir = TempDir::new().unwrap();
        let paths = paths_in(&dir);
        let _open = source_with_uncheckpointed_rows(&paths.source, 5);
        assert!(
            paths
                .source
                .with_extension("sqlite-wal")
                .metadata()
                .unwrap()
                .len()
                > 0
        );

        let snapshot = take(&paths).unwrap();

        assert_eq!(count_rows(&snapshot.path), 5);
        assert!(snapshot.bytes > 0);
    }

    #[test]
    fn the_snapshot_is_one_self_contained_file() {
        let dir = TempDir::new().unwrap();
        let paths = paths_in(&dir);
        let _open = source_with_uncheckpointed_rows(&paths.source, 1);

        take(&paths).unwrap();

        let mut names: Vec<_> = fs::read_dir(&paths.dest_dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        names.sort();
        assert_eq!(names, ["latest.sqlite"]);
    }

    #[test]
    fn a_new_snapshot_atomically_replaces_the_old_one() {
        let dir = TempDir::new().unwrap();
        let paths = paths_in(&dir);
        let source = source_with_uncheckpointed_rows(&paths.source, 2);
        take(&paths).unwrap();
        assert_eq!(count_rows(&paths.snapshot()), 2);

        source
            .execute("INSERT INTO activity (app) VALUES ('later')", [])
            .unwrap();
        take(&paths).unwrap();

        assert_eq!(count_rows(&paths.snapshot()), 3);
        assert!(!paths.temp().exists());
    }

    #[test]
    fn a_stale_temp_file_does_not_corrupt_the_snapshot() {
        let dir = TempDir::new().unwrap();
        let paths = paths_in(&dir);
        let _open = source_with_uncheckpointed_rows(&paths.source, 2);
        fs::create_dir_all(&paths.dest_dir).unwrap();
        fs::write(paths.temp(), b"garbage from a crashed run").unwrap();

        take(&paths).unwrap();

        assert_eq!(count_rows(&paths.snapshot()), 2);
    }

    #[test]
    fn a_missing_source_is_an_error_and_leaves_the_old_snapshot() {
        let dir = TempDir::new().unwrap();
        let paths = paths_in(&dir);
        fs::create_dir_all(&paths.dest_dir).unwrap();
        fs::write(paths.snapshot(), b"previous").unwrap();

        let error = take(&paths).unwrap_err();

        assert!(matches!(error, SnapshotError::SourceMissing(_)));
        assert_eq!(fs::read(paths.snapshot()).unwrap(), b"previous");
    }

    #[test]
    fn an_unreadable_source_fails_without_replacing_the_snapshot() {
        let dir = TempDir::new().unwrap();
        let paths = paths_in(&dir);
        fs::write(
            &paths.source,
            b"this is not a sqlite database at all, just text",
        )
        .unwrap();
        fs::create_dir_all(&paths.dest_dir).unwrap();
        fs::write(paths.snapshot(), b"previous").unwrap();

        assert!(take(&paths).is_err());

        assert_eq!(fs::read(paths.snapshot()).unwrap(), b"previous");
        assert!(!paths.temp().exists());
    }
}
