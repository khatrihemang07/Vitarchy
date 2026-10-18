//! A card Statement result and a Transaction total, shared by the confirm tests.

use super::{
    app::Env,
    results::{card_section, read},
};
use vitarchy_finance::reader::{ReaderResult, contract::Transaction};

/// The mask every confirm test's card prints.
pub const MASK: &str = "4123XXXXXXXX2003";

/// A read result with one card section holding `rows`.
pub fn card(rows: Vec<Transaction>) -> ReaderResult {
    read(vec![card_section(MASK, rows)])
}

/// How many Transactions are stored in all.
pub async fn transaction_total(env: &Env) -> i64 {
    env.get("/transactions").await.json["total"]
        .as_i64()
        .unwrap()
}
