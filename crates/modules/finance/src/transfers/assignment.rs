//! One payment and the receipts assigned to it.

/// Receipts assigned to the Transfer of `debit_id`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    /// The card-bill payment.
    pub debit_id: i64,
    /// The receipts that paid it.
    pub receipt_ids: Vec<i64>,
}

/// What a pass assigned, and how many payments or receipts it left alone as ambiguous.
#[derive(Debug, Default)]
pub struct Assigned {
    /// The assignments.
    pub assigned: Vec<Assignment>,
    /// Left alone because several fit.
    pub ambiguous: usize,
}
