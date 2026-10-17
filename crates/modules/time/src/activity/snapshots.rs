//! Every Source's Snapshot file, resolved once and shared by the routes.

use std::{collections::HashMap, ffi::OsString, path::PathBuf};

use super::{snapshot_path::SnapshotFile, source::Source};

/// One [`SnapshotFile`] per [`Source`].
#[derive(Debug, Clone)]
pub struct Snapshots(HashMap<Source, SnapshotFile>);

impl Snapshots {
    /// Resolves each Source's file from its env var (read through `env`) or its default under `home`.
    pub fn resolve(home: Option<&std::path::Path>, env: impl Fn(&str) -> Option<OsString>) -> Self {
        Self(
            Source::all()
                .map(|source| {
                    let descriptor = source.descriptor();
                    let file = SnapshotFile::resolve(
                        home,
                        env(descriptor.snapshot_env),
                        descriptor.default_relative,
                    );
                    (source, file)
                })
                .collect(),
        )
    }

    /// Resolves from the process environment and `HOME`.
    #[must_use]
    pub fn from_env() -> Self {
        Self::resolve(
            std::env::var_os("HOME").map(PathBuf::from).as_deref(),
            |name| std::env::var_os(name),
        )
    }

    /// Replaces one Source's file (tests point a Source at a fixture).
    #[must_use]
    pub fn with(mut self, source: Source, file: SnapshotFile) -> Self {
        self.0.insert(source, file);
        self
    }

    /// The file for `source`.
    #[must_use]
    pub fn get(&self, source: Source) -> &SnapshotFile {
        // `resolve` fills every Source, and `with` only replaces.
        &self.0[&source]
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn each_source_gets_its_own_env_var_and_default() {
        let env =
            |name: &str| (name == "TIME_CLOCKIFY_SNAPSHOT").then(|| OsString::from("/c.sqlite"));
        let snapshots = Snapshots::resolve(Some(Path::new("/h")), env);
        assert_eq!(
            snapshots.get(Source::Clockify).path(),
            Path::new("/c.sqlite")
        );
        assert_eq!(
            snapshots.get(Source::Toggl).path(),
            Path::new("/h/Library/Application Support/Vitarchy/activity/latest.sqlite")
        );
    }

    #[test]
    fn with_replaces_only_the_named_source() {
        let snapshots = Snapshots::resolve(Some(Path::new("/h")), |_| None)
            .with(Source::Toggl, SnapshotFile::at("/fixture.sqlite"));
        assert_eq!(
            snapshots.get(Source::Toggl).path(),
            Path::new("/fixture.sqlite")
        );
        assert!(snapshots.get(Source::Clockify).path().starts_with("/h"));
    }
}
