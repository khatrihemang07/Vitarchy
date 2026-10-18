//! Builders for the payment-app join tests: a bank file and an app file for one Instrument,
//! both confirmed on one savings Account.

use super::{
    app::Env,
    results::{history, instrument, read, section},
    tx,
};
use vitarchy_finance::{
    reader::contract::{AccountKind, Transaction},
    repo::accounts,
};

pub const UTR: &str = "512345678901";

pub fn bank_row(day: u32) -> Transaction {
    let mut row = tx(
        day,
        "UPI/DR/512345678901/SOME MERCHANT/HDFC/xyautopay",
        25_000,
    );
    row.reference = Some(UTR.to_owned());
    row
}

pub fn app_file(rows: Vec<Transaction>) -> vitarchy_finance::reader::ReaderResult {
    history(vec![instrument(None, "XXXXXX4321", rows)])
}

pub fn bank_file(rows: Vec<Transaction>) -> vitarchy_finance::reader::ReaderResult {
    read(vec![section(
        Some("SBI"),
        Some(AccountKind::Savings),
        "XXXXXX4321",
        rows,
    )])
}

/// An Env whose reader answers `answers` in order, with the SBI savings Account the
/// instrument `XXXXXX4321` belongs to.
pub async fn confirm_all(label: &str, answers: Vec<vitarchy_finance::reader::ReaderResult>) -> Env {
    let env = Env::new(label, answers).await;
    accounts::insert(
        &env.db.pool,
        "SBI",
        AccountKind::Savings,
        Some("XXXXXX4321"),
        "Salary",
    )
    .await
    .unwrap();
    env
}

pub async fn confirm_next(env: &Env, name: &str) -> i64 {
    let id = env.upload_one(name, name.as_bytes()).await;
    let reply = env.confirm_proposed(id, 1).await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    id
}

pub async fn printings_of_the_only_transaction(env: &Env) -> usize {
    let page = env.get("/transactions").await;
    assert_eq!(page.json["total"], 1, "one payment is one Transaction");
    let id = page.json["items"][0]["id"].as_i64().unwrap();
    env.get(&format!("/transactions/{id}/printings"))
        .await
        .json
        .as_array()
        .unwrap()
        .len()
}
