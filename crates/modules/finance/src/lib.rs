//! Finance: the Module that turns the person's Statements into exact Transactions
//! (ADR 0012-0014). Owns the `finance` Postgres schema. Files are read by the
//! statement-reader helper; this crate stores, dedupes and links what it returns.

mod accounts;
mod dedupe;
mod error;
pub mod files;
mod import;
mod linking;
mod problem;
pub mod reader;
mod record_key;
pub mod repo;
mod routes;
#[cfg(test)]
mod test_support;
mod view;

use vitarchy_core::Module;

pub use accounts::{AccountIdentity, Match, find as match_account, last4, same_number};
pub use dedupe::{Evidence, ReadAgainDiff, StoredEvidence, new_rows, read_again_diff};
pub use error::FinanceError;
pub use linking::{RecordFacts, TransactionFacts, link};
pub use routes::router_with;

/// The Finance Module, mounted at `/api/finance`.
#[must_use]
pub fn module() -> Module {
    Module::new("finance", routes::router, sqlx::migrate!("./migrations"))
}
