//! Pure: turns Toggl's Time Entries into the writes a Sync makes.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::toggl::types::{Raw, TogglEntry};

/// A Time Entry ready to store.
#[derive(Debug, Clone, PartialEq)]
pub struct EntryRow {
    /// Toggl's id (primary key).
    pub toggl_id: i64,
    /// Owning workspace.
    pub workspace_id: i64,
    /// Project, if any.
    pub project_id: Option<i64>,
    /// Task, if any.
    pub task_id: Option<i64>,
    /// What the person typed (empty when none).
    pub description: String,
    /// Tag names.
    pub tags: Vec<String>,
    /// Billable flag.
    pub billable: bool,
    /// Start time.
    pub start: DateTime<Utc>,
    /// Stop time (always set: running entries are not stored).
    pub stop: DateTime<Utc>,
    /// Length in seconds.
    pub duration_s: i64,
    /// Toggl's last-modified time.
    pub toggl_updated_at: Option<DateTime<Utc>>,
    /// The original record.
    pub raw: Value,
}

/// What one Sync writes.
#[derive(Debug, Default, PartialEq)]
pub struct Plan {
    /// Entries to insert or update, by Toggl id.
    pub upserts: Vec<EntryRow>,
    /// Toggl ids deleted in Toggl, to delete here.
    pub deletes: Vec<i64>,
    /// Entries still running, left for a later Sync.
    pub skipped_running: u32,
}

/// Plans the writes for `entries`. Deleted entries win over everything else; a
/// running entry (no stop, or a negative duration) is skipped until it stops.
/// An id listed twice keeps its last occurrence, so replaying a payload changes nothing.
#[must_use]
pub fn plan(entries: Vec<Raw<TogglEntry>>, default_workspace_id: i64) -> Plan {
    let mut upserts = BTreeMap::new();
    let mut deletes = BTreeSet::new();
    let mut skipped_running = BTreeSet::new();
    for Raw { item, raw } in entries {
        let id = item.id;
        upserts.remove(&id);
        deletes.remove(&id);
        skipped_running.remove(&id);
        if item.server_deleted_at.is_some() {
            deletes.insert(id);
        } else if let Some(row) = stored_row(item, raw, default_workspace_id) {
            upserts.insert(id, row);
        } else {
            skipped_running.insert(id);
        }
    }
    Plan {
        upserts: upserts.into_values().collect(),
        deletes: deletes.into_iter().collect(),
        skipped_running: u32::try_from(skipped_running.len()).unwrap_or(u32::MAX),
    }
}

fn stored_row(entry: TogglEntry, raw: Value, default_workspace_id: i64) -> Option<EntryRow> {
    let stop = entry.stop.filter(|_| entry.duration >= 0)?;
    Some(EntryRow {
        toggl_id: entry.id,
        workspace_id: entry.workspace_id.unwrap_or(default_workspace_id),
        project_id: entry.project_id,
        task_id: entry.task_id,
        description: entry.description.unwrap_or_default(),
        tags: entry.tags.unwrap_or_default(),
        billable: entry.billable.unwrap_or(false),
        start: entry.start,
        stop,
        duration_s: entry.duration,
        toggl_updated_at: entry.at,
        raw,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn entry(json: Value) -> Raw<TogglEntry> {
        Raw {
            item: serde_json::from_value(json.clone()).unwrap(),
            raw: json,
        }
    }

    fn stopped(id: i64) -> Raw<TogglEntry> {
        entry(json!({
            "id": id, "workspace_id": 7, "project_id": 3, "description": "write",
            "tags": ["a"], "billable": true, "start": "2026-04-01T10:00:00+00:00",
            "stop": "2026-04-01T11:00:00+00:00", "duration": 3600,
            "at": "2026-04-01T11:00:01+00:00"
        }))
    }

    #[test]
    fn a_stopped_entry_becomes_an_upsert_with_its_raw_record() {
        let planned = plan(vec![stopped(1)], 99);
        assert_eq!(planned.upserts.len(), 1);
        let row = &planned.upserts[0];
        assert_eq!(
            (row.toggl_id, row.workspace_id, row.duration_s),
            (1, 7, 3600)
        );
        assert_eq!(row.tags, ["a"]);
        assert_eq!(row.raw["description"], "write");
    }

    #[test]
    fn missing_optional_fields_get_defaults() {
        let planned = plan(
            vec![entry(json!({
                "id": 2, "start": "2026-04-01T10:00:00Z",
                "stop": "2026-04-01T10:00:05Z", "duration": 5
            }))],
            99,
        );
        let row = &planned.upserts[0];
        assert_eq!(row.workspace_id, 99);
        assert_eq!((row.description.as_str(), row.billable), ("", false));
        assert!(row.tags.is_empty());
    }

    #[test]
    fn a_running_entry_is_skipped_and_counted() {
        let running = entry(json!({
            "id": 3, "start": "2026-04-01T10:00:00Z", "stop": null, "duration": -1_775_037_600
        }));
        let planned = plan(vec![running, stopped(1)], 99);
        assert_eq!(planned.skipped_running, 1);
        assert_eq!(planned.upserts.len(), 1);
    }

    #[test]
    fn a_negative_duration_is_running_even_with_a_stop_time() {
        let odd = entry(json!({
            "id": 4, "start": "2026-04-01T10:00:00Z", "stop": "2026-04-01T11:00:00Z", "duration": -5
        }));
        assert_eq!(plan(vec![odd], 99).skipped_running, 1);
    }

    #[test]
    fn an_entry_deleted_in_toggl_becomes_a_delete_not_an_upsert() {
        let deleted = entry(json!({
            "id": 5, "start": "2026-04-01T10:00:00Z", "stop": "2026-04-01T11:00:00Z",
            "duration": 3600, "server_deleted_at": "2026-04-02T00:00:00Z"
        }));
        let planned = plan(vec![deleted], 99);
        assert_eq!(planned.deletes, [5]);
        assert!(planned.upserts.is_empty());
    }

    #[test]
    fn replaying_the_same_payload_gives_the_same_plan() {
        let payload = || vec![stopped(1), stopped(2)];
        assert_eq!(plan(payload(), 99), plan(payload(), 99));
    }

    #[test]
    fn an_id_listed_twice_keeps_only_its_last_occurrence() {
        let deleted = entry(json!({
            "id": 1, "start": "2026-04-01T10:00:00Z", "duration": 1,
            "server_deleted_at": "2026-04-02T00:00:00Z"
        }));
        let planned = plan(vec![stopped(1), deleted], 99);
        assert_eq!(planned.deletes, [1]);
        assert!(planned.upserts.is_empty());
    }
}
