//! Whether a Statement's row would make a stored Transaction show more detail (ADR 0015).
//! The one definition: preview, Read again and confirm all count "gains detail" through it.

use super::{Printing, resolve};
use crate::reader::contract::Transaction;

/// Whether adding `incoming` to `existing` would change what the Transaction shows.
#[must_use]
pub fn would_gain(existing: &[Printing], incoming: &Printing) -> bool {
    let mut all = existing.to_vec();
    all.push(incoming.clone());
    resolve(existing).detail != resolve(&all).detail
}

/// Whether Statement `statement_id` printing `tx` would add detail to the stored Transaction
/// whose Printings are `existing` (`None` when it has none). A Statement that already printed
/// the Transaction gains nothing: it has no new Printing to add.
#[must_use]
pub fn gains_detail(statement_id: i64, tx: &Transaction, existing: Option<&[Printing]>) -> bool {
    existing.is_some_and(|list| {
        list.iter().all(|p| p.statement_id != statement_id)
            && would_gain(list, &Printing::of(statement_id, tx))
    })
}
