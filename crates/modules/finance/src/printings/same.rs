//! When two Printings of one Transaction are the same (ADR 0015). The one definition:
//! confirm, preview, Read again and Account merge all skip a Printing through it.

use super::Printing;

/// Whether `a` and `b` print the Transaction exactly alike (equal [`super::PrintedFields`]).
/// Which Statement printed them, and whether they are gone, do not matter.
#[must_use]
pub fn same_as(a: &Printing, b: &Printing) -> bool {
    a.printed == b.printed
}

/// Whether a kept, live Printing among `kept` is `same_as` the `incoming` one.
#[must_use]
pub fn matches_live(kept: &[Printing], incoming: &Printing) -> bool {
    kept.iter().any(|p| !p.gone && same_as(p, incoming))
}

/// The ids of the `from` Printings that are identical to a live one in `into`: a merge
/// of two Transactions drops those instead of moving them.
#[must_use]
pub fn identical_to_live(from: &[Printing], into: &[Printing]) -> Vec<i64> {
    from.iter()
        .filter(|p| matches_live(into, p))
        .map(|p| p.id)
        .collect()
}
