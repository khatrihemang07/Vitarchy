//! Repairing duplicate Accounts: Accounts with the same issuer, kind and visible digits are
//! merged into the oldest, folding overlapping Transactions into one with several Printings.
//! Nothing is lost: only the emptied Account rows (and the Transaction rows folded into a
//! kept one, whose Printings move across) are deleted.

mod apply;
mod groups;
mod plan;
mod repair;

pub use repair::{MergeReport, merge_duplicate_accounts};

#[cfg(test)]
mod groups_tests;
#[cfg(test)]
mod plan_tests;
