//! What an Import did, as shown to the person and stored in `runs.counts`.

use serde::Serialize;
use utoipa::ToSchema;

/// Result of one Import.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct ImportCounts {
    /// Rows found in the Snapshot.
    pub read: u32,
    /// Activity stored for the first time.
    pub inserted: u32,
    /// Activity already stored whose fields changed (e.g. a later end).
    pub updated: u32,
    /// Activity already stored and identical.
    pub unchanged: u32,
    /// Rows left out: unreadable id, impossible times, or a repeated id.
    pub skipped: u32,
}
