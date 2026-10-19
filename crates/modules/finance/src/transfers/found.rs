//! What one rung hands back to the ladder.

use super::{Link, Rung, sole_partners::mutual};

/// The links one rung found, and the debits it left alone because they fit several partners.
#[derive(Debug, Default)]
pub struct Found {
    /// New links, in debit order.
    pub links: Vec<Link>,
    /// Debits or credits left alone as ambiguous.
    pub ambiguous: usize,
}

impl Found {
    /// The links of every (debit, credit) pair where neither has another candidate partner;
    /// debits with several are counted as ambiguous.
    #[must_use]
    pub fn from_unique_pairs(pairs: &[(i64, i64)], rung: Rung) -> Self {
        let (kept, ambiguous) = mutual(pairs);
        Self {
            links: pairs_to_links(kept, rung),
            ambiguous,
        }
    }
}

/// One single-credit link per pair, ordered by debit.
#[must_use]
pub fn pairs_to_links(mut pairs: Vec<(i64, i64)>, rung: Rung) -> Vec<Link> {
    pairs.sort_unstable();
    pairs
        .into_iter()
        .map(|(debit_id, credit_id)| Link {
            debit_id,
            credit_ids: vec![credit_id],
            rung,
        })
        .collect()
}
