//! The rows of one preview section: which are new, which are Transactions already stored,
//! and which of those would gain detail from this Statement's Printing.

use std::collections::HashMap;

use super::transaction::TransactionView;
use crate::{
    dedupe::{Origin, StoredEvidence, pair},
    printings::{Plan, Printing, plan_for},
    reader::contract::Transaction,
};

/// A section's rows and counts.
pub struct Rows {
    /// The rows in printed order.
    pub views: Vec<TransactionView>,
    /// Rows that would become new Transactions.
    pub new_count: usize,
    /// Rows that would gain detail.
    pub enriched_count: usize,
}

/// Compares `incoming` (printed by Statement `statement_id`, a file of `origin`) with the Account's `stored`
/// Transactions and their `printings`. A row whose Statement already printed it gains nothing.
pub fn build(
    statement_id: i64,
    origin: Origin,
    incoming: &[Transaction],
    stored: &[StoredEvidence],
    printings: &HashMap<i64, Vec<Printing>>,
) -> Rows {
    let mut rows = Rows {
        views: Vec::with_capacity(incoming.len()),
        new_count: 0,
        enriched_count: 0,
    };
    for (tx, found) in incoming.iter().zip(pair(incoming, origin, stored)) {
        let gains = found.is_some_and(|id| {
            plan_for(statement_id, tx, printings.get(&id).map(Vec::as_slice))
                == Plan::Add { gains: true }
        });
        rows.new_count += usize::from(found.is_none());
        rows.enriched_count += usize::from(gains);
        rows.views
            .push(TransactionView::of(tx, found.is_some(), gains));
    }
    rows
}
