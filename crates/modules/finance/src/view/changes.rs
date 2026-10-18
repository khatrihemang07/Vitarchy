//! What a Read again would change, as the API returns it.

use serde::Serialize;
use utoipa::ToSchema;

use super::{model::StatementPreview, transaction::TransactionView};
use crate::{
    import::read_again_diff::{Diff, SectionDiff},
    reader::ReaderResult,
};

/// The changes to one Account section.
#[derive(Serialize, ToSchema)]
pub struct SectionChanges {
    /// The section's printed order.
    pub position: i32,
    /// Rows that would be added.
    pub add_count: usize,
    /// Stored rows the engine still finds.
    pub unchanged_count: usize,
    /// Stored rows the engine no longer finds: marked, never deleted.
    pub no_longer_found_count: usize,
    /// The rows that would be added.
    pub added: Vec<TransactionView>,
}

/// The changes to a Payment history.
#[derive(Serialize, ToSchema)]
pub struct RecordChanges {
    /// Records that would be added.
    pub add_count: usize,
    /// Records already stored.
    pub unchanged_count: usize,
}

/// A Read again: the Statement as it is now, and what reading it again would change.
#[derive(Serialize, ToSchema)]
pub struct ReadAgainPreview {
    /// The Statement as stored (an unconfirmed one is already re-read).
    pub statement: StatementPreview,
    /// Per Account section; empty for an unconfirmed Statement or a Payment history.
    pub sections: Vec<SectionChanges>,
    /// For a confirmed Payment history.
    pub payment_records: Option<RecordChanges>,
}

/// One Statement's part of Read all again: its preview, or why it could not be read again.
#[derive(Serialize, ToSchema)]
pub struct ReadAgainItem {
    /// The Statement.
    pub statement_id: i64,
    /// The uploaded file's name.
    pub file_name: String,
    /// What reading it again would change; absent when it failed.
    pub preview: Option<ReadAgainPreview>,
    /// Why it failed (a problem code such as `statement_not_readable`).
    pub error_code: Option<String>,
}

fn section(diff: &SectionDiff, result: &ReaderResult) -> SectionChanges {
    let incoming = usize::try_from(diff.position)
        .ok()
        .and_then(|index| result.statements.get(index))
        .map_or(&[][..], |s| s.transactions.as_slice());
    SectionChanges {
        position: diff.position,
        add_count: diff.add.len(),
        unchanged_count: diff.unchanged.len(),
        no_longer_found_count: diff.no_longer_found.len(),
        added: diff
            .add
            .iter()
            .filter_map(|index| incoming.get(*index))
            .map(|tx| TransactionView::of(tx, false))
            .collect(),
    }
}

/// The changes `diff` describes, for the file `result` it was computed from.
#[must_use]
pub fn of_diff(diff: &Diff, result: &ReaderResult) -> (Vec<SectionChanges>, Option<RecordChanges>) {
    match diff {
        Diff::Sections(sections) => (sections.iter().map(|d| section(d, result)).collect(), None),
        Diff::Records { add, unchanged } => (
            vec![],
            Some(RecordChanges {
                add_count: add.len(),
                unchanged_count: *unchanged,
            }),
        ),
    }
}
