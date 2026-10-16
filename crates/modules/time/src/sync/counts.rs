//! What a Sync did, as shown to the person and stored in `runs.counts`.

use serde::Serialize;
use utoipa::ToSchema;

/// Result of one Sync.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct SyncCounts {
    /// Time Entries stored for the first time.
    pub inserted: u32,
    /// Time Entries that were already stored and were written again.
    pub updated: u32,
    /// Time Entries removed because they were deleted in Toggl.
    pub deleted: u32,
    /// Running timers left for a later Sync.
    pub skipped_running: u32,
    /// Projects in the workspace.
    pub projects: u32,
    /// Clients in the workspace.
    pub clients: u32,
    /// Tags in the workspace.
    pub tags: u32,
}

/// A list length as a count (saturating, never panicking).
pub fn count(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}
