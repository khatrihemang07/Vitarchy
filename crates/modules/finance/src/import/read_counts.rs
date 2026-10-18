//! What a read found, for the run log: counts only, never content.

use serde::Serialize;

use crate::reader::ReaderResult;

/// Sizes of a read result.
#[derive(Serialize, Default)]
pub struct ReadCounts {
    sections: usize,
    transactions: usize,
}

impl ReadCounts {
    /// The sizes of `result`.
    #[must_use]
    pub fn of(result: &ReaderResult) -> Self {
        Self {
            sections: result.statements.len(),
            transactions: result.statements.iter().map(|s| s.transactions.len()).sum(),
        }
    }
}
