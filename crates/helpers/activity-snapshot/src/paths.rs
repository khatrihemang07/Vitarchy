//! Where the Snapshot is read from and written to. Pure: no filesystem access.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Env var that replaces the source database (for tests and stage).
pub const SOURCE_ENV: &str = "ACTIVITY_SNAPSHOT_SOURCE";
/// Env var that replaces the destination folder (for tests and stage).
pub const DEST_DIR_ENV: &str = "ACTIVITY_SNAPSHOT_DEST_DIR";

/// The Toggl Track for Mac database, relative to the home folder. The single
/// place to correct if Activity turns out to live elsewhere.
const SOURCE_RELATIVE: &str = "Library/Group Containers/B227VTMZ94.group.com.toggl.daneel.extensions/production/DatabaseModel.sqlite";
/// Vitarchy's Activity folder, relative to the home folder. With [`SNAPSHOT_FILE`] it is
/// restated in the time Module and `scripts/lib/helper.sh`; `scripts/test-guards.sh`
/// fails if they differ.
const DEST_DIR_RELATIVE: &str = "Library/Application Support/Vitarchy/activity";
/// The Snapshot's file name. Its modification time is the Snapshot time.
pub const SNAPSHOT_FILE: &str = "latest.sqlite";

/// The resolved source and destination of one run.
#[derive(Debug, PartialEq, Eq)]
pub struct Paths {
    /// The database to copy.
    pub source: PathBuf,
    /// The folder the Snapshot is written into.
    pub dest_dir: PathBuf,
}

impl Paths {
    /// Resolves the paths from the home folder, letting `env` override either one.
    pub fn resolve(home: &Path, env: impl Fn(&str) -> Option<OsString>) -> Self {
        let pick = |name: &str, relative: &str| {
            env(name)
                .filter(|value| !value.is_empty())
                .map_or_else(|| home.join(relative), PathBuf::from)
        };
        Self {
            source: pick(SOURCE_ENV, SOURCE_RELATIVE),
            dest_dir: pick(DEST_DIR_ENV, DEST_DIR_RELATIVE),
        }
    }

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

    fn no_env(_: &str) -> Option<OsString> {
        None
    }

    #[test]
    fn defaults_live_under_the_home_folder() {
        let paths = Paths::resolve(Path::new("/Users/me"), no_env);
        assert_eq!(
            paths.source,
            Path::new(
                "/Users/me/Library/Group Containers/B227VTMZ94.group.com.toggl.daneel.extensions/production/DatabaseModel.sqlite"
            )
        );
        assert_eq!(
            paths.snapshot(),
            Path::new("/Users/me/Library/Application Support/Vitarchy/activity/latest.sqlite")
        );
    }

    #[test]
    fn the_temp_file_sits_beside_the_snapshot() {
        let paths = Paths::resolve(Path::new("/h"), no_env);
        assert_eq!(paths.temp().parent(), paths.snapshot().parent());
        assert_eq!(paths.temp().file_name().unwrap(), "latest.sqlite.tmp");
    }

    #[test]
    fn env_overrides_each_path() {
        let env = |name: &str| match name {
            SOURCE_ENV => Some(OsString::from("/x/src.sqlite")),
            DEST_DIR_ENV => Some(OsString::from("/x/out")),
            _ => None,
        };
        let paths = Paths::resolve(Path::new("/h"), env);
        assert_eq!(paths.source, Path::new("/x/src.sqlite"));
        assert_eq!(paths.snapshot(), Path::new("/x/out/latest.sqlite"));
    }

    #[test]
    fn an_empty_override_is_ignored() {
        let env = |_: &str| Some(OsString::new());
        let paths = Paths::resolve(Path::new("/h"), env);
        assert!(paths.source.starts_with("/h/Library"));
    }
}
