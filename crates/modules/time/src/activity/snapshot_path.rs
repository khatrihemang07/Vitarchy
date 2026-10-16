//! Where the Snapshot lives. The helper writes it; its modification time is its age.

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    time::SystemTime,
};

use chrono::{DateTime, Utc};

/// Env var that replaces the Snapshot path (stage and tests).
pub const SNAPSHOT_ENV: &str = "TIME_ACTIVITY_SNAPSHOT";
/// Where the helper writes the Snapshot, relative to the home folder.
const DEFAULT_RELATIVE: &str = "Library/Application Support/Vitarchy/activity/latest.sqlite";

/// The Snapshot file's location (it may not exist yet).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotFile(PathBuf);

impl SnapshotFile {
    /// A Snapshot at an exact path.
    #[must_use]
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    /// The path from `override_path` when it is non-empty, else under `home`.
    /// With neither, a relative path that will not exist, so the state is "no Snapshot".
    #[must_use]
    pub fn resolve(home: Option<&Path>, override_path: Option<OsString>) -> Self {
        match (override_path.filter(|value| !value.is_empty()), home) {
            (Some(path), _) => Self(PathBuf::from(path)),
            (None, Some(home)) => Self(home.join(DEFAULT_RELATIVE)),
            (None, None) => Self(PathBuf::from(DEFAULT_RELATIVE)),
        }
    }

    /// Resolves from the process environment (`TIME_ACTIVITY_SNAPSHOT`, `HOME`).
    #[must_use]
    pub fn from_env() -> Self {
        Self::resolve(
            std::env::var_os("HOME").map(PathBuf::from).as_deref(),
            std::env::var_os(SNAPSHOT_ENV),
        )
    }

    /// The file's path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.0
    }

    /// When the Snapshot was taken (the file's modification time), or `None` when absent.
    pub async fn taken_at(&self) -> Option<DateTime<Utc>> {
        let modified = tokio::fs::metadata(&self.0).await.ok()?.modified().ok()?;
        Some(to_utc(modified))
    }
}

fn to_utc(time: SystemTime) -> DateTime<Utc> {
    time.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_env_override_wins_over_the_home_default() {
        let file = SnapshotFile::resolve(Some(Path::new("/h")), Some("/tmp/s.sqlite".into()));
        assert_eq!(file.path(), Path::new("/tmp/s.sqlite"));
    }

    #[test]
    fn without_an_override_it_sits_in_the_application_support_folder() {
        let file = SnapshotFile::resolve(Some(Path::new("/h")), Some(OsString::new()));
        assert_eq!(
            file.path(),
            Path::new("/h/Library/Application Support/Vitarchy/activity/latest.sqlite")
        );
    }

    #[tokio::test]
    async fn a_missing_file_has_no_taken_time() {
        assert!(
            SnapshotFile::at("/nonexistent/x.sqlite")
                .taken_at()
                .await
                .is_none()
        );
    }
}
