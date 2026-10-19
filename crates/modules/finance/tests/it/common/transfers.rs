//! Statements of several invented Accounts, imported in order, for the Transfer tests. Every
//! name, number and amount here is made up.

use serde_json::Value;
use vitarchy_finance::reader::{
    ReaderResult,
    contract::{AccountKind, Transaction},
};

use super::{
    app::Env,
    results::{card_section, read, section},
};

/// A file of one savings Account ending in `mask`.
pub fn savings_file(mask: &str, rows: Vec<Transaction>) -> ReaderResult {
    read(vec![section(
        Some("Example Bank"),
        Some(AccountKind::Savings),
        mask,
        rows,
    )])
}

/// A file of one loan Account ending in `mask`.
pub fn loan_file(mask: &str, rows: Vec<Transaction>) -> ReaderResult {
    read(vec![section(
        Some("Example Bank"),
        Some(AccountKind::Loan),
        mask,
        rows,
    )])
}

/// A file of one card ending in `mask`.
pub fn card_file(mask: &str, rows: Vec<Transaction>) -> ReaderResult {
    read(vec![card_section(mask, rows)])
}

/// Uploads the next file (named `name`, whose bytes are its name) and confirms it as proposed.
/// Returns the Statement id.
pub async fn import(env: &Env, name: &str) -> i64 {
    let id = env
        .upload_one(&format!("{name}.pdf"), name.as_bytes())
        .await;
    let reply = env.confirm_proposed(id, 1).await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    id
}

/// Every Transaction row, as the list returns them.
pub async fn items(env: &Env) -> Vec<Value> {
    env.get("/transactions?limit=500").await.json["items"]
        .as_array()
        .unwrap()
        .clone()
}

/// The row whose description is exactly `description`.
pub async fn row(env: &Env, description: &str) -> Value {
    items(env)
        .await
        .into_iter()
        .find(|item| item["description"] == description)
        .unwrap_or_else(|| panic!("no row {description}"))
}

/// The `transfer` of the row described `description` (null when it has none).
pub async fn transfer_of(env: &Env, description: &str) -> Value {
    row(env, description).await["transfer"].clone()
}

/// How many Transfers are stored.
pub async fn transfer_count(env: &Env) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM finance.transfers")
        .fetch_one(&env.db.pool)
        .await
        .unwrap()
}
