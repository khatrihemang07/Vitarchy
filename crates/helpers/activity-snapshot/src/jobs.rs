//! The Snapshot jobs one run performs: one per Source, resolved from the home folder.
//! Variation between Sources is data in [`SPECS`].

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::newest::{Entry, list_entries, newest_matching};
use crate::paths::{CLOCKIFY_DEST_DIR_RELATIVE, Paths, TOGGL_DEST_DIR_RELATIVE};

/// How to find a Source's database.
enum Locate {
    /// One known file, relative to the home folder.
    File(&'static str),
    /// The newest `<prefix>…<suffix>` in a folder, relative to the home folder.
    Newest {
        dir: &'static str,
        prefix: &'static str,
        suffix: &'static str,
    },
}

/// Everything that differs between Sources.
struct Spec {
    /// The Source's id, as the server names it.
    source: &'static str,
    /// Env var that replaces the source database (tests and stage).
    source_env: &'static str,
    /// Env var that replaces the destination folder (tests and stage).
    dest_env: &'static str,
    dest_dir_relative: &'static str,
    locate: Locate,
}

/// The Toggl Track for Mac database. The single place to correct if Activity lives elsewhere.
const TOGGL_SOURCE_RELATIVE: &str = "Library/Group Containers/B227VTMZ94.group.com.toggl.daneel.extensions/production/DatabaseModel.sqlite";
const CLOCKIFY_SOURCE_DIR_RELATIVE: &str = "Library/Application Support/Clockify Desktop";

/// Every Source, in the order the server lists them.
const SPECS: [Spec; 2] = [
    Spec {
        source: "toggl",
        source_env: "ACTIVITY_SNAPSHOT_SOURCE",
        dest_env: "ACTIVITY_SNAPSHOT_DEST_DIR",
        dest_dir_relative: TOGGL_DEST_DIR_RELATIVE,
        locate: Locate::File(TOGGL_SOURCE_RELATIVE),
    },
    Spec {
        source: "clockify",
        source_env: "CLOCKIFY_SNAPSHOT_SOURCE",
        dest_env: "CLOCKIFY_SNAPSHOT_DEST_DIR",
        dest_dir_relative: CLOCKIFY_DEST_DIR_RELATIVE,
        locate: Locate::Newest {
            dir: CLOCKIFY_SOURCE_DIR_RELATIVE,
            prefix: "Clockify_",
            suffix: ".sqlite",
        },
    },
];

/// One Snapshot to take.
#[derive(Debug, PartialEq, Eq)]
pub struct Job {
    /// The Source's id, for the log line.
    pub source: &'static str,
    /// Where to read and write.
    pub paths: Paths,
}

/// One job per Source. `env` overrides paths; `list` lists a folder (the filesystem, or a
/// fake in tests). When a folder has no matching file the source is the folder itself,
/// which is not a file, so the job fails with "source database not found" naming it.
pub fn resolve_jobs(
    home: &Path,
    env: impl Fn(&str) -> Option<OsString>,
    list: impl Fn(&Path) -> Vec<Entry>,
) -> Vec<Job> {
    let pick = |name: &str| {
        env(name)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    };
    SPECS
        .iter()
        .map(|spec| Job {
            source: spec.source,
            paths: Paths {
                source: pick(spec.source_env).unwrap_or_else(|| locate(home, &spec.locate, &list)),
                dest_dir: pick(spec.dest_env).unwrap_or_else(|| home.join(spec.dest_dir_relative)),
            },
        })
        .collect()
}

/// [`resolve_jobs`] against the real filesystem.
#[must_use]
pub fn resolve_jobs_on_disk(home: &Path, env: impl Fn(&str) -> Option<OsString>) -> Vec<Job> {
    resolve_jobs(home, env, list_entries)
}

fn locate(home: &Path, locate: &Locate, list: &impl Fn(&Path) -> Vec<Entry>) -> PathBuf {
    match locate {
        Locate::File(relative) => home.join(relative),
        Locate::Newest {
            dir,
            prefix,
            suffix,
        } => {
            let dir = home.join(dir);
            match newest_matching(&list(&dir), prefix, suffix) {
                Some(entry) => dir.join(&entry.name),
                None => dir,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use super::*;

    fn no_env(_: &str) -> Option<OsString> {
        None
    }

    fn entry(name: &str) -> Entry {
        Entry {
            name: name.into(),
            modified: SystemTime::UNIX_EPOCH,
        }
    }

    fn jobs(env: impl Fn(&str) -> Option<OsString>, files: Vec<Entry>) -> Vec<Job> {
        resolve_jobs(Path::new("/Users/me"), env, move |_| files.clone())
    }

    #[test]
    fn there_is_one_job_per_source_in_the_servers_order() {
        let ids: Vec<_> = jobs(no_env, vec![]).iter().map(|job| job.source).collect();
        assert_eq!(ids, ["toggl", "clockify"]);
    }

    #[test]
    fn defaults_live_under_the_home_folder() {
        let jobs = jobs(no_env, vec![entry("Clockify_42.sqlite")]);
        assert_eq!(
            jobs[0].paths.source,
            Path::new(
                "/Users/me/Library/Group Containers/B227VTMZ94.group.com.toggl.daneel.extensions/production/DatabaseModel.sqlite"
            )
        );
        assert_eq!(
            jobs[0].paths.snapshot(),
            Path::new("/Users/me/Library/Application Support/Vitarchy/activity/latest.sqlite")
        );
        assert_eq!(
            jobs[1].paths.source,
            Path::new("/Users/me/Library/Application Support/Clockify Desktop/Clockify_42.sqlite")
        );
        assert_eq!(
            jobs[1].paths.snapshot(),
            Path::new("/Users/me/Library/Application Support/Vitarchy/clockify/latest.sqlite")
        );
    }

    #[test]
    fn with_no_clockify_database_the_source_is_the_folder() {
        let jobs = jobs(no_env, vec![entry("Preferences.plist")]);
        assert_eq!(
            jobs[1].paths.source,
            Path::new("/Users/me/Library/Application Support/Clockify Desktop")
        );
    }

    #[test]
    fn each_source_has_its_own_env_overrides() {
        let env = |name: &str| match name {
            "ACTIVITY_SNAPSHOT_SOURCE" => Some(OsString::from("/x/toggl.sqlite")),
            "ACTIVITY_SNAPSHOT_DEST_DIR" => Some(OsString::from("/x/toggl-out")),
            "CLOCKIFY_SNAPSHOT_SOURCE" => Some(OsString::from("/x/clk.sqlite")),
            "CLOCKIFY_SNAPSHOT_DEST_DIR" => Some(OsString::from("/x/clk-out")),
            _ => None,
        };
        let jobs = jobs(env, vec![]);
        assert_eq!(jobs[0].paths.source, Path::new("/x/toggl.sqlite"));
        assert_eq!(
            jobs[0].paths.snapshot(),
            Path::new("/x/toggl-out/latest.sqlite")
        );
        assert_eq!(jobs[1].paths.source, Path::new("/x/clk.sqlite"));
        assert_eq!(
            jobs[1].paths.snapshot(),
            Path::new("/x/clk-out/latest.sqlite")
        );
    }

    #[test]
    fn an_empty_override_is_ignored() {
        let jobs = jobs(|_| Some(OsString::new()), vec![]);
        assert!(jobs[0].paths.source.starts_with("/Users/me/Library"));
    }
}
