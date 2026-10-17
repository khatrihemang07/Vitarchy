//! Where each Snapshot is read from and written to. Pure: no filesystem access.

use std::path::PathBuf;

/// The Snapshot's file name, the same for every Source. Its modification time is the Snapshot time.
pub const SNAPSHOT_FILE: &str = "latest.sqlite";
/// Toggl's Snapshot folder, relative to the home folder. With [`SNAPSHOT_FILE`] it is
/// restated in the time Module and `scripts/lib/helper.sh`; `scripts/test-guards.sh`
/// fails if they differ.
pub const TOGGL_DEST_DIR_RELATIVE: &str = "Library/Application Support/Vitarchy/activity";
/// Clockify's Snapshot folder, restated and guarded the same way.
pub const CLOCKIFY_DEST_DIR_RELATIVE: &str = "Library/Application Support/Vitarchy/clockify";

/// The source and destination of one Snapshot.
#[derive(Debug, PartialEq, Eq)]
pub struct Paths {
    /// The database to copy.
    pub source: PathBuf,
    /// The folder the Snapshot is written into.
    pub dest_dir: PathBuf,
}

impl Paths {
    /// The finished Snapshot file.
    #[must_use]
    pub fn snapshot(&self) -> PathBuf {
        self.dest_dir.join(SNAPSHOT_FILE)
    }

    /// The in-progress copy, in the same folder so the final rename is atomic.
    #[must_use]
    pub fn temp(&self) -> PathBuf {
        self.dest_dir.join(format!("{SNAPSHOT_FILE}.tmp"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths() -> Paths {
        Paths {
            source: PathBuf::from("/s.sqlite"),
            dest_dir: PathBuf::from("/d"),
        }
    }

    #[test]
    fn the_snapshot_is_latest_sqlite_in_the_destination() {
        assert_eq!(paths().snapshot(), PathBuf::from("/d/latest.sqlite"));
    }

    #[test]
    fn the_temp_file_sits_beside_the_snapshot() {
        let paths = paths();
        assert_eq!(paths.temp().parent(), paths.snapshot().parent());
        assert_eq!(paths.temp().file_name().unwrap(), "latest.sqlite.tmp");
    }
}
