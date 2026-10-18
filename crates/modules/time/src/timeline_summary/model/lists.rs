//! The Day's Segments and Time Entries.

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{activity::Source, timeline::model::Away};

/// A Segment clipped to the Period.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct SegmentView {
    /// When it began.
    pub start_at: DateTime<Utc>,
    /// When it ended.
    pub end_at: DateTime<Utc>,
    /// The frontmost app.
    pub app: String,
    /// The window title.
    pub title: String,
    /// The page address, when a Source recorded one.
    pub url: Option<String>,
    /// Whether the person was away.
    pub away: Away,
    /// Every Source that covered it.
    pub sources: Vec<Source>,
}

/// A Time Entry on the Day lane, clipped to the Day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct LaneEntry {
    /// When it began.
    pub start_at: DateTime<Utc>,
    /// When it stopped.
    pub stop_at: DateTime<Utc>,
    /// What the person typed; may be empty.
    pub description: String,
    /// The project's name, when known.
    pub project: Option<String>,
}
