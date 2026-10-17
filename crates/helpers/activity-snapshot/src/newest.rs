//! Choosing which file in a tracking app's folder is its current database.
//!
//! Clockify Desktop names its database `Clockify_<account id>.sqlite`, so the right file
//! depends on who is signed in. The newest one is taken: the signed-in account is the one
//! the app is writing to.

use std::fs;
use std::path::Path;
use std::time::SystemTime;

/// A file in a folder: its name and when it was last modified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// File name, without the folder.
    pub name: String,
    /// Last modification time.
    pub modified: SystemTime,
}

/// The newest entry named `<prefix>…<suffix>`. A tie goes to the greater name so the pick
/// is stable. Write-ahead-log companions (`….sqlite-wal`) do not end in the suffix.
#[must_use]
pub fn newest_matching<'a>(entries: &'a [Entry], prefix: &str, suffix: &str) -> Option<&'a Entry> {
    entries
        .iter()
        .filter(|entry| entry.name.starts_with(prefix) && entry.name.ends_with(suffix))
        .max_by(|a, b| (a.modified, &a.name).cmp(&(b.modified, &b.name)))
}

/// The files directly inside `dir`. An unreadable or missing folder has none, which
/// surfaces later as "source database not found".
#[must_use]
pub fn list_entries(dir: &Path) -> Vec<Entry> {
    let Ok(read) = fs::read_dir(dir) else {
        return Vec::new();
    };
    read.filter_map(|entry| {
        let entry = entry.ok()?;
        let modified = entry.metadata().ok()?.modified().ok()?;
        Some(Entry {
            name: entry.file_name().into_string().ok()?,
            modified,
        })
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn entry(name: &str, age_secs: u64) -> Entry {
        Entry {
            name: name.into(),
            modified: SystemTime::UNIX_EPOCH + Duration::from_secs(1_000 - age_secs),
        }
    }

    #[test]
    fn the_most_recently_modified_match_wins() {
        let entries = [
            entry("Clockify_old.sqlite", 500),
            entry("Clockify_new.sqlite", 5),
            entry("Clockify_mid.sqlite", 50),
        ];
        let picked = newest_matching(&entries, "Clockify_", ".sqlite").unwrap();
        assert_eq!(picked.name, "Clockify_new.sqlite");
    }

    #[test]
    fn other_files_and_wal_companions_are_ignored_even_when_newer() {
        let entries = [
            entry("Clockify_a.sqlite", 100),
            entry("Clockify_a.sqlite-wal", 1),
            entry("Clockify_a.sqlite-shm", 1),
            entry("Preferences.plist", 1),
        ];
        let picked = newest_matching(&entries, "Clockify_", ".sqlite").unwrap();
        assert_eq!(picked.name, "Clockify_a.sqlite");
    }

    #[test]
    fn nothing_matching_is_none() {
        assert!(newest_matching(&[entry("x.sqlite", 1)], "Clockify_", ".sqlite").is_none());
        assert!(newest_matching(&[], "Clockify_", ".sqlite").is_none());
    }

    #[test]
    fn equal_times_pick_the_greater_name() {
        let entries = [entry("Clockify_a.sqlite", 5), entry("Clockify_b.sqlite", 5)];
        let picked = newest_matching(&entries, "Clockify_", ".sqlite").unwrap();
        assert_eq!(picked.name, "Clockify_b.sqlite");
    }

    #[test]
    fn a_missing_folder_has_no_entries() {
        assert!(list_entries(Path::new("/nonexistent/vitarchy")).is_empty());
    }
}
