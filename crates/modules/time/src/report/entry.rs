//! The Time Entry as a Report reads it: the stored entry plus its project's name and client.

use chrono::{DateTime, Utc};

/// A stopped Time Entry with the project facts a Report needs.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct WindowEntry {
    /// Toggl's id for the entry.
    pub toggl_id: i64,
    /// The project, when it has one.
    pub project_id: Option<i64>,
    /// The project's name, when the project is known.
    pub project_name: Option<String>,
    /// The project's client, when it has one.
    pub client_id: Option<i64>,
    /// What the person typed; may be empty.
    pub description: String,
    /// Tag names.
    pub tags: Vec<String>,
    /// When it started.
    pub start: DateTime<Utc>,
    /// When it stopped.
    pub stop: DateTime<Utc>,
}

#[cfg(test)]
pub mod fixtures {
    use super::*;

    /// A one-hour entry at `start` (RFC 3339) with the given description; other fields empty.
    pub fn entry(id: i64, start: &str, hours: i64, description: &str) -> WindowEntry {
        let start: DateTime<Utc> = start.parse().unwrap();
        WindowEntry {
            toggl_id: id,
            project_id: None,
            project_name: None,
            client_id: None,
            description: description.to_owned(),
            tags: vec![],
            start,
            stop: start + chrono::Duration::hours(hours),
        }
    }
}
