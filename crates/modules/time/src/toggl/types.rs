//! Shapes of Toggl's JSON. Only the fields Vitarchy reads are typed; the whole
//! record is kept beside it (`Raw::raw`) so later analyses can use the rest.

use chrono::{DateTime, NaiveDate, Utc};
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::error::TogglError;

/// `GET /me`: who the token belongs to.
#[derive(Debug, serde::Deserialize)]
pub struct Me {
    /// Toggl user id.
    pub id: i64,
    /// The workspace Vitarchy syncs.
    pub default_workspace_id: i64,
}

/// `GET /workspaces/{id}`.
#[derive(Debug, serde::Deserialize)]
pub struct Workspace {
    /// Workspace name, shown on the status page.
    pub name: String,
}

/// One Time Entry as Toggl returns it.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TogglEntry {
    /// Toggl's id for the entry.
    pub id: i64,
    /// Owning workspace (absent in some payloads).
    pub workspace_id: Option<i64>,
    /// Project the entry belongs to, if any.
    pub project_id: Option<i64>,
    /// Task the entry belongs to, if any.
    pub task_id: Option<i64>,
    /// What the person typed; may be null.
    pub description: Option<String>,
    /// Tag names; may be null.
    pub tags: Option<Vec<String>>,
    /// Whether the entry is billable.
    pub billable: Option<bool>,
    /// When the entry started.
    pub start: DateTime<Utc>,
    /// When it stopped; null while the timer runs.
    pub stop: Option<DateTime<Utc>>,
    /// Seconds; negative while the timer runs.
    pub duration: i64,
    /// Last modification time on Toggl's side.
    pub at: Option<DateTime<Utc>>,
    /// Set when the entry was deleted in Toggl (shown by `since` queries).
    pub server_deleted_at: Option<DateTime<Utc>>,
}

/// A project in the workspace.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TogglProject {
    /// Toggl's id.
    pub id: i64,
    /// Owning client, if any.
    pub client_id: Option<i64>,
    /// Project name.
    pub name: String,
    /// False once archived.
    pub active: Option<bool>,
    /// Display colour as Toggl stores it.
    pub color: Option<String>,
}

/// A client in the workspace.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TogglClient {
    /// Toggl's id.
    pub id: i64,
    /// Client name.
    pub name: String,
    /// True once archived.
    pub archived: Option<bool>,
}

/// A tag in the workspace.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TogglTag {
    /// Toggl's id.
    pub id: i64,
    /// Tag name.
    pub name: String,
}

/// A parsed record together with the JSON it came from.
#[derive(Debug, Clone)]
pub struct Raw<T> {
    /// The fields Vitarchy reads.
    pub item: T,
    /// The original record, stored as `raw jsonb`.
    pub raw: Value,
}

impl<T: DeserializeOwned> Raw<T> {
    /// Parses a JSON array (or `null`, which Toggl sends for "none") into records.
    ///
    /// # Errors
    /// [`TogglError::Malformed`] when the value is not a list of valid records.
    pub fn parse_list(value: Value) -> Result<Vec<Self>, TogglError> {
        match value {
            Value::Null => Ok(Vec::new()),
            Value::Array(items) => items.into_iter().map(Self::parse).collect(),
            _ => Err(TogglError::Malformed),
        }
    }

    fn parse(raw: Value) -> Result<Self, TogglError> {
        let item = serde_json::from_value(raw.clone()).map_err(|_| TogglError::Malformed)?;
        Ok(Self { item, raw })
    }
}

/// Which Time Entries to ask Toggl for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryQuery {
    /// Everything between two dates (first Sync).
    Range {
        /// First day included.
        start_date: NaiveDate,
        /// Day after the last one included.
        end_date: NaiveDate,
    },
    /// Everything changed or deleted since a Unix time (later Syncs).
    Since(i64),
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn a_list_keeps_each_record_beside_its_typed_fields() {
        let list =
            Raw::<TogglTag>::parse_list(json!([{"id": 1, "name": "deep", "extra": true}])).unwrap();
        assert_eq!(list[0].item.name, "deep");
        assert_eq!(list[0].raw["extra"], true);
    }

    #[test]
    fn null_means_an_empty_list() {
        assert!(Raw::<TogglTag>::parse_list(Value::Null).unwrap().is_empty());
    }

    #[test]
    fn a_record_of_the_wrong_shape_is_malformed() {
        let result = Raw::<TogglTag>::parse_list(json!([{"id": "x"}]));
        assert_eq!(result.unwrap_err(), TogglError::Malformed);
        let result = Raw::<TogglTag>::parse_list(json!({"id": 1}));
        assert_eq!(result.unwrap_err(), TogglError::Malformed);
    }

    #[test]
    fn an_entry_with_null_optionals_parses() {
        let entry: TogglEntry = serde_json::from_value(json!({
            "id": 9, "start": "2026-01-01T10:00:00+00:00", "stop": null,
            "duration": -1, "description": null, "tags": null
        }))
        .unwrap();
        assert!(entry.stop.is_none() && entry.description.is_none());
    }
}
