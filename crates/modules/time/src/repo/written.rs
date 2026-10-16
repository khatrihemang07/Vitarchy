//! The outcome of a batched upsert, shared by every table that reports it.

/// How many rows of a batch were new and how many changed.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Written {
    /// Rows stored for the first time.
    pub inserted: u64,
    /// Existing rows whose fields were written again.
    pub updated: u64,
}

impl Written {
    /// From the `(inserted, updated)` counts a batch statement returns (`xmax = 0` marks a new row).
    #[must_use]
    pub fn from_counts((inserted, updated): (i64, i64)) -> Self {
        Self {
            inserted: u64::try_from(inserted).unwrap_or(0),
            updated: u64::try_from(updated).unwrap_or(0),
        }
    }
}
