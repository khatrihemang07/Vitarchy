//! The one Activity shape every Source's parser produces.

use chrono::{DateTime, Utc};
use serde_json::Value;

use super::source::Source;

/// What only some Sources record. A Source leaves the others' fields empty.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Extras {
    /// Toggl: the device that recorded it, when readable.
    pub client_id: Option<String>,
    /// Toggl: whether the computer was idle.
    pub idle: Option<bool>,
    /// Clockify: the page address, for browsers.
    pub url: Option<String>,
    /// Clockify: idle seconds as reported, even when longer than the span.
    pub idle_secs: Option<f64>,
}

/// An Activity ready to store, exactly as its Source recorded it.
#[derive(Debug, Clone, PartialEq)]
pub struct Activity {
    /// Which Source recorded it.
    pub source: Source,
    /// The Source's id for it; unique within the Source.
    pub source_id: String,
    /// App name.
    pub app: String,
    /// Window title (may be empty).
    pub title: String,
    /// When it began.
    pub start_at: DateTime<Utc>,
    /// When it ended so far.
    pub end_at: DateTime<Utc>,
    /// Every source column (never a Snapshot-only blob such as an icon).
    pub raw: Value,
    /// What only some Sources record.
    pub extras: Extras,
}

/// The result of parsing a whole Snapshot.
#[derive(Debug, Default, PartialEq)]
pub struct Parsed {
    /// Distinct, valid Activity.
    pub activities: Vec<Activity>,
    /// Rows left out.
    pub skipped: usize,
}
