//! Finance: the Module that turns the person's Statements into exact Transactions
//! (ADR 0012-0016). Owns the `finance` Postgres schema. Files are read by the
//! statement-reader helper; this crate stores and dedupes what it returns.

mod accounts;
mod dedupe;
mod error;
pub mod files;
mod import;
mod linking;
mod merge;
mod printings;
mod problem;
pub mod reader;
pub mod repo;
mod routes;
#[cfg(test)]
mod test_support;
mod transfers;
mod view;

use vitarchy_core::Module;

pub use accounts::{
    AccountIdentity, Match, find as match_account, find_instrument, instrument_key, last4,
    same_number,
};
pub use dedupe::{
    Evidence, Origin, Origins, PrintingToAdd, ReadAgainDiff, StoredEvidence, new_rows, pair,
    read_again_diff,
};
pub use error::FinanceError;
pub use linking::{BACKFILL_MARKER, backfill_once};
pub use merge::{MergeReport, merge_duplicate_accounts};
pub use routes::router_with;

/// The Finance Module, mounted at `/api/finance`.
#[must_use]
pub fn module() -> Module {
    Module::new("finance", routes::router, sqlx::migrate!("./migrations"))
        .with_startup(|pool| Box::pin(async move { linking::backfill_on_start(&pool).await }))
}
