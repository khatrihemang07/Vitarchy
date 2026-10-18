//! Builders for the payment-app Instrument tests: Accounts, one-row histories, confirm calls.

use super::{
    app::{Env, Reply},
    results::{history, instrument},
    tx,
};
use vitarchy_finance::{reader::contract::AccountKind, repo::accounts};

pub async fn account(
    env: &Env,
    issuer: &str,
    kind: AccountKind,
    mask: Option<&str>,
    name: &str,
) -> i64 {
    accounts::insert(&env.db.pool, issuer, kind, mask, name)
        .await
        .unwrap()
        .id
}

pub fn file(issuer: Option<&str>, mask: &str, day: u32) -> vitarchy_finance::reader::ReaderResult {
    history(vec![instrument(
        issuer,
        mask,
        vec![tx(day, "Paid to SHOP", 100 + i64::from(day))],
    )])
}

pub async fn confirm(env: &Env, name: &str, body: serde_json::Value) -> (i64, Reply) {
    let id = env.upload_one(name, name.as_bytes()).await;
    let reply = env
        .post(&format!("/statements/{id}/confirm"), &body.to_string())
        .await;
    (id, reply)
}
