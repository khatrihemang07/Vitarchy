//! Where one Snapshot lives. The helper writes it; its modification time is its age.

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    time::SystemTime,
};

use chrono::{DateTime, Utc};

/// The Snapshot file's location (it may not exist yet).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotFile(PathBuf);

impl SnapshotFile {
    /// A Snapshot at an exact path.
    #[must_use]
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    /// A Snapshot at the path from `override_path` when it is non-empty, else `relative`
    /// under `home` (see [`resolve_path`]).
    #[must_use]
    pub fn resolve(home: Option<&Path>, override_path: Option<OsString>, relative: &str) -> Self {
        Self(resolve_path(home, override_path, relative))
    }

    /// The file's path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.0
    }

    /// When the Snapshot was taken (the file's modification time), or `None` when absent.
    pub async fn taken_at(&self) -> Option<DateTime<Utc>> {
        modified_at(&self.0).await
    }
}

/// `override_path` when it is non-empty, else `relative` under `home`. With neither,
/// `relative` itself, which will not exist, so the state is "no Snapshot".
fn resolve_path(home: Option<&Path>, override_path: Option<OsString>, relative: &str) -> PathBuf {
    match (override_path.filter(|value| !value.is_empty()), home) {
        (Some(path), _) => PathBuf::from(path),
        (None, Some(home)) => home.join(relative),
        (None, None) => PathBuf::from(relative),
    }
}

/// When the file at `path` was last modified, or `None` when absent.
async fn modified_at(path: &Path) -> Option<DateTime<Utc>> {
    let modified = tokio::fs::metadata(path).await.ok()?.modified().ok()?;
    Some(to_utc(modified))
}

fn to_utc(time: SystemTime) -> DateTime<Utc> {
    time.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    const RELATIVE: &str = "Library/x/latest.sqlite";

    #[test]
    fn the_override_wins_over_the_home_default() {
        let file = SnapshotFile::resolve(
            Some(Path::new("/h")),
            Some("/tmp/s.sqlite".into()),
            RELATIVE,
        );
        assert_eq!(file.path(), Path::new("/tmp/s.sqlite"));
    }

    #[test]
    fn without_an_override_it_sits_under_the_home_folder() {
        let file = SnapshotFile::resolve(Some(Path::new("/h")), Some(OsString::new()), RELATIVE);
        assert_eq!(file.path(), Path::new("/h/Library/x/latest.sqlite"));
    }

    #[test]
    fn without_a_home_the_relative_path_is_used_and_will_not_exist() {
        let file = SnapshotFile::resolve(None, None, RELATIVE);
        assert_eq!(file.path(), Path::new(RELATIVE));
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
