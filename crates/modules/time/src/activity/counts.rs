//! What an Import did, as shown to the person and stored in `runs.counts`.

use serde::Serialize;
use utoipa::ToSchema;

use crate::{repo::written::Written, saturating::saturating_u32};

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

impl ImportCounts {
    /// From `read` Snapshot rows, `stored` parsed Activity, what the upsert `written`, and `skipped` rows.
    #[must_use]
    pub fn tally(read: usize, stored: usize, written: &Written, skipped: usize) -> Self {
        let (inserted, updated) = (
            saturating_u32(written.inserted),
            saturating_u32(written.updated),
        );
        Self {
            read: saturating_u32(read),
            inserted,
            updated,
            unchanged: saturating_u32(stored)
                .saturating_sub(inserted)
                .saturating_sub(updated),
            skipped: saturating_u32(skipped),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whatever_was_not_written_is_unchanged() {
        let written = Written {
            inserted: 2,
            updated: 1,
            range: None,
        };
        let counts = ImportCounts::tally(10, 8, &written, 2);
        assert_eq!((counts.read, counts.unchanged, counts.skipped), (10, 5, 2));
    }
}
