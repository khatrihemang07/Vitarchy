//! Pure: turns Toggl's Time Entries into the writes a Sync makes.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Duration, NaiveDate, NaiveTime, Utc};
use serde_json::Value;

use crate::{
    saturating::saturating_u32,
    toggl::types::{EntryQuery, Raw, TogglEntry},
};

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

/// Mirrored entries to remove because Toggl no longer lists them (Range syncs only).
#[derive(Debug, PartialEq, Eq)]
pub struct Reconcile {
    /// First instant covered (inclusive).
    pub from: DateTime<Utc>,
    /// Last instant covered (exclusive).
    pub to: DateTime<Utc>,
    /// Ids Toggl listed as stopped entries; every other stored entry starting in the span goes.
    pub keep: Vec<i64>,
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
    /// Entries of other workspaces (`/me/time_entries` spans all of them), ignored.
    pub skipped_other_workspace: u32,
    /// For a Range sync, which stored entries Toggl no longer has.
    pub reconcile: Option<Reconcile>,
}

/// Days trimmed from each edge of a Range window before reconciling: Toggl reads
/// the dates in its own time zone, so entries at the very edge may be missing
/// from the answer without being deleted.
const RECONCILE_MARGIN_DAYS: i64 = 1;

/// Plans the writes for `entries`, which answer `query` for `workspace_id`.
///
/// - Entries of other workspaces are ignored.
/// - Deleted entries win over everything else; a running entry (no stop, or a
///   negative duration) is skipped until it stops.
/// - An id listed twice keeps its last occurrence, so replaying a payload changes nothing.
/// - A Range answer is a complete snapshot of its window, so it also reconciles:
///   stored entries starting inside it but absent from it are deleted. That
///   includes an entry that is now running, which comes back when it stops (the
///   mirror holds stopped entries only, whether or not Toggl lists the runner).
///   A `Since` answer lists only changes, so it never reconciles.
#[must_use]
pub fn plan(entries: Vec<Raw<TogglEntry>>, workspace_id: i64, query: &EntryQuery) -> Plan {
    let mut upserts = BTreeMap::new();
    let mut deletes = BTreeSet::new();
    let mut skipped_running = BTreeSet::new();
    let mut foreign = BTreeSet::new();
    for Raw { item, raw } in entries {
        let id = item.id;
        upserts.remove(&id);
        deletes.remove(&id);
        skipped_running.remove(&id);
        foreign.remove(&id);
        if item.workspace_id.is_some_and(|owner| owner != workspace_id) {
            foreign.insert(id);
        } else if item.server_deleted_at.is_some() {
            deletes.insert(id);
        } else if let Some(row) = stored_row(item, raw, workspace_id) {
            upserts.insert(id, row);
        } else {
            skipped_running.insert(id);
        }
    }
    let reconcile = reconcile_for(query, upserts.keys().copied().collect());
    Plan {
        upserts: upserts.into_values().collect(),
        deletes: deletes.into_iter().collect(),
        skipped_running: saturating_u32(skipped_running.len()),
        skipped_other_workspace: saturating_u32(foreign.len()),
        reconcile,
    }
}

fn reconcile_for(query: &EntryQuery, keep: Vec<i64>) -> Option<Reconcile> {
    let EntryQuery::Range {
        start_date,
        end_date,
    } = query
    else {
        return None;
    };
    let edge = |date: &NaiveDate, days: i64| {
        (date.and_time(NaiveTime::MIN).and_utc()) + Duration::days(days)
    };
    Some(Reconcile {
        from: edge(start_date, RECONCILE_MARGIN_DAYS),
        to: edge(end_date, -RECONCILE_MARGIN_DAYS),
        keep,
    })
}

fn stored_row(entry: TogglEntry, raw: Value, workspace_id: i64) -> Option<EntryRow> {
    let stop = entry.stop.filter(|_| entry.duration >= 0)?;
    Some(EntryRow {
        toggl_id: entry.id,
        workspace_id: entry.workspace_id.unwrap_or(workspace_id),
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

    use chrono::{NaiveDate, TimeZone};

    use super::*;

    fn since() -> EntryQuery {
        EntryQuery::Since(0)
    }

    fn range() -> EntryQuery {
        EntryQuery::Range {
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 4, 12).unwrap(),
        }
    }

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
        let planned = plan(vec![stopped(1)], 7, &since());
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
            &since(),
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
        let planned = plan(vec![running, stopped(1)], 7, &since());
        assert_eq!(planned.skipped_running, 1);
        assert_eq!(planned.upserts.len(), 1);
    }

    #[test]
    fn a_negative_duration_is_running_even_with_a_stop_time() {
        let odd = entry(json!({
            "id": 4, "start": "2026-04-01T10:00:00Z", "stop": "2026-04-01T11:00:00Z", "duration": -5
        }));
        assert_eq!(plan(vec![odd], 7, &since()).skipped_running, 1);
    }

    #[test]
    fn an_entry_deleted_in_toggl_becomes_a_delete_not_an_upsert() {
        let deleted = entry(json!({
            "id": 5, "start": "2026-04-01T10:00:00Z", "stop": "2026-04-01T11:00:00Z",
            "duration": 3600, "server_deleted_at": "2026-04-02T00:00:00Z"
        }));
        let planned = plan(vec![deleted], 7, &since());
        assert_eq!(planned.deletes, [5]);
        assert!(planned.upserts.is_empty());
    }

    #[test]
    fn replaying_the_same_payload_gives_the_same_plan() {
        let payload = || vec![stopped(1), stopped(2)];
        assert_eq!(plan(payload(), 7, &since()), plan(payload(), 7, &since()));
    }

    #[test]
    fn an_id_listed_twice_keeps_only_its_last_occurrence() {
        let deleted = entry(json!({
            "id": 1, "start": "2026-04-01T10:00:00Z", "duration": 1,
            "server_deleted_at": "2026-04-02T00:00:00Z"
        }));
        let planned = plan(vec![stopped(1), deleted], 7, &since());
        assert_eq!(planned.deletes, [1]);
        assert!(planned.upserts.is_empty());
    }

    #[test]
    fn an_entry_of_another_workspace_is_ignored_and_counted() {
        let foreign = entry(json!({
            "id": 8, "workspace_id": 555, "start": "2026-04-01T10:00:00Z",
            "stop": "2026-04-01T11:00:00Z", "duration": 3600
        }));
        let foreign_deleted = entry(json!({
            "id": 9, "workspace_id": 555, "start": "2026-04-01T10:00:00Z", "duration": 1,
            "server_deleted_at": "2026-04-02T00:00:00Z"
        }));
        let planned = plan(vec![foreign, foreign_deleted, stopped(1)], 7, &since());
        assert_eq!(planned.upserts.len(), 1);
        assert!(planned.deletes.is_empty());
        assert_eq!(planned.skipped_other_workspace, 2);
    }

    #[test]
    fn a_since_sync_does_not_reconcile() {
        assert_eq!(plan(vec![stopped(1)], 7, &since()).reconcile, None);
    }

    #[test]
    fn a_range_sync_reconciles_inside_the_window_keeping_only_stored_entries() {
        let running = entry(json!({
            "id": 3, "workspace_id": 7, "start": "2026-04-01T10:00:00Z", "stop": null, "duration": -5
        }));
        let planned = plan(vec![stopped(2), stopped(1), running], 7, &range());
        let reconcile = planned.reconcile.expect("a range sync reconciles");
        // One day inside each edge: Toggl reads the dates in its own time zone.
        assert_eq!(
            reconcile.from,
            Utc.with_ymd_and_hms(2026, 1, 2, 0, 0, 0).unwrap()
        );
        assert_eq!(
            reconcile.to,
            Utc.with_ymd_and_hms(2026, 4, 11, 0, 0, 0).unwrap()
        );
        // A running entry is not kept: the mirror holds stopped entries only.
        assert_eq!(reconcile.keep, [1, 2]);
    }
}
