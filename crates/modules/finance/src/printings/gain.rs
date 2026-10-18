//! Whether a Statement's row adds a Printing, and whether it would show more detail (ADR 0015).
//! The one definition: preview, Read again and confirm all count through it.

use super::{Printing, resolve, same::matches_live};
use crate::reader::contract::Transaction;

/// What a Statement printing a stored Transaction does to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    /// Nothing to store: this Statement already printed the Transaction.
    ThisStatement,
    /// Nothing to store: a live, kept Printing of another Statement is exactly the same.
    IdenticalKept,
    /// A new Printing is stored; `gains` when it changes what the Transaction shows.
    Add {
        /// Whether the shown view changes.
        gains: bool,
    },
}

/// Whether adding `incoming` to `existing` would change what the Transaction shows.
#[must_use]
pub fn would_gain(existing: &[Printing], incoming: &Printing) -> bool {
    let mut all = existing.to_vec();
    all.push(incoming.clone());
    resolve(existing).detail != resolve(&all).detail
}

/// What Statement `statement_id` printing `tx` does to the stored Transaction whose Printings
/// are `existing` (`None` when it has none).
#[must_use]
pub fn plan_for(statement_id: i64, tx: &Transaction, existing: Option<&[Printing]>) -> Plan {
    let list = existing.unwrap_or_default();
    let incoming = Printing::of(statement_id, tx);
    if list.iter().any(|p| p.statement_id == statement_id) {
        return Plan::ThisStatement;
    }
    if matches_live(list, &incoming) {
        return Plan::IdenticalKept;
    }
    Plan::Add {
        gains: existing.is_some() && would_gain(list, &incoming),
    }
}
