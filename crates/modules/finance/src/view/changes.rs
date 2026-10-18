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
    /// Rows this Statement printed that the engine still finds.
    pub unchanged_count: usize,
    /// Rows that match a Transaction another Statement stored and would gain a Printing.
    pub new_printing_count: usize,
    /// Of those, Printings that would add detail to the Transaction.
    pub enriched_count: usize,
    /// Printings the engine no longer finds: marked, never deleted.
    pub no_longer_found_count: usize,
    /// The rows that would be added.
    pub added: Vec<TransactionView>,
    /// The rows that are stored Transactions and would gain a Printing, tagged when it
    /// would add detail.
    pub new_printings: Vec<TransactionView>,
}

/// A Read again: the Statement as it is now, and what reading it again would change.
#[derive(Serialize, ToSchema)]
pub struct ReadAgainPreview {
    /// The Statement as stored (an unconfirmed one is already re-read).
    pub statement: StatementPreview,
    /// Per Account section (a payment app's instrument is a section too); empty for an
    /// unconfirmed Statement.
    pub sections: Vec<SectionChanges>,
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
        new_printing_count: diff.new_printings.len(),
        enriched_count: diff.gaining.len(),
        no_longer_found_count: diff.gone_transaction_ids.len(),
        added: diff
            .add
            .iter()
            .filter_map(|index| incoming.get(*index))
            .map(|tx| TransactionView::of(tx, false, false))
            .collect(),
        new_printings: diff
            .new_printings
            .iter()
            .filter_map(|p| incoming.get(p.index).map(|tx| (p.index, tx)))
            .map(|(index, tx)| TransactionView::of(tx, true, diff.gaining.contains(&index)))
            .collect(),
    }
}

/// The changes `diff` describes, for the file `result` it was computed from.
#[must_use]
pub fn of_diff(diff: &Diff, result: &ReaderResult) -> Vec<SectionChanges> {
    diff.sections.iter().map(|d| section(d, result)).collect()
}
