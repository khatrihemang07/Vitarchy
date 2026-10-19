//! Transfers: money moving between two of the person's own Accounts (ADR 0017). Pure
//! detection, strongest evidence first; the database side lives in [`crate::linking`].
//!
//! A Transfer is one Money-out Transaction and one or more Money-in Transactions on other
//! Accounts. Detection pairs Transactions on a ladder of rungs, each needing exact amounts
//! except card-bill payments:
//!
//! 1. reference: the debit's reference appears in the credit, within a day;
//! 2. account number: the debit names the other Account's number, the same day;
//! 3. bill payment: a card-bill payment and the receipts on cards it paid, within a day,
//!    where the credits may exceed the debit by up to [`BILL_TOLERANCE_PAISE`] and may
//!    still be arriving (a partial Transfer);
//! 4. unique amount: the only debit and credit of that amount around that day.
//!
//! A Transaction is used by one rung only, a pair the person unlinked is never linked again,
//! and anything that fits several partners is left alone.

mod account_number;
mod assignment;
mod bill_payment;
mod context;
mod figures;
mod found;
mod ladder;
mod markers;
mod model;
mod place_leftover_receipts;
mod reference;
mod settle_whole_payments;
mod sole_partners;
mod subsets;
mod summary;
mod tokens;
mod unique_amount;
mod validate;

pub use figures::{Figures, figures};
pub use ladder::detect;
pub use model::{
    AccountInfo, BILL_TOLERANCE_PAISE, Candidate, Detection, ExistingTransfer, Input, Link, Rung,
};
pub use summary::Parts;
pub use validate::{Invalid, fits, validate_manual};

#[cfg(test)]
mod account_number_tests;
#[cfg(test)]
mod bill_payment_tests;
#[cfg(test)]
mod ladder_tests;
#[cfg(test)]
mod reference_tests;
#[cfg(test)]
mod support;
#[cfg(test)]
mod unique_amount_tests;
