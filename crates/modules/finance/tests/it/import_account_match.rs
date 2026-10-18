//! Which Account a section is proposed for: trailing digits, and no printed number.

use crate::common::{
    app::Env,
    results::{read, section, without_number},
    tx,
};
use serde_json::json;
use vitarchy_finance::{
    reader::contract::{AccountKind, Transaction},
    repo::accounts,
};

fn uni(mask: Option<&str>, rows: Vec<Transaction>) -> vitarchy_finance::reader::ReaderResult {
    let found = section(Some("Uni"), Some(AccountKind::CreditCard), "x", rows);
    read(vec![match mask {
        Some(mask) => section(
            Some("Uni"),
            Some(AccountKind::CreditCard),
            mask,
            found.transactions,
        ),
        None => without_number(found),
    }])
}

async fn add_card(env: &Env, mask: &str) {
    accounts::insert(
        &env.db.pool,
        "Uni",
        AccountKind::CreditCard,
        Some(mask),
        mask,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn a_file_with_no_number_matches_the_only_account_of_that_issuer_and_kind() {
    let env = Env::new("am_only", vec![uni(None, vec![tx(2, "A", 100)])]).await;
    add_card(&env, "XXXX XXXX XXXX 9876").await;
    let id = env.upload_one("a.pdf", b"a").await;

    let preview = env.get(&format!("/statements/{id}")).await;
    let reply = env.confirm_proposed(id, 1).await;

    let section = &preview.json["sections"][0];
    assert_eq!(section["matched_without_number"], true);
    assert!(!section["matched_account"].is_null());
    assert_eq!(
        reply.json,
        json!({"added": 1, "duplicates": 0, "linked": 0})
    );
    assert_eq!(env.get("/accounts").await.json.as_array().unwrap().len(), 1);
    env.finish().await;
}

#[tokio::test]
async fn a_file_with_no_number_and_several_accounts_needs_a_choice() {
    let env = Env::new("am_many", vec![uni(None, vec![tx(2, "A", 100)])]).await;
    add_card(&env, "XXXX9876").await;
    add_card(&env, "XXXX2468").await;
    let id = env.upload_one("a.pdf", b"a").await;

    let preview = env.get(&format!("/statements/{id}")).await;
    let reply = env.confirm_proposed(id, 1).await;

    let section = &preview.json["sections"][0];
    assert_eq!(section["matched_without_number"], false);
    assert_eq!(section["candidate_accounts"].as_array().unwrap().len(), 2);
    assert_eq!(reply.json["code"], "account_ambiguous");
    env.finish().await;
}

#[tokio::test]
async fn a_numbered_match_is_not_flagged_as_matched_without_a_number() {
    let env = Env::new(
        "am_numbered",
        vec![uni(Some("xxxx xxxx xxxx 9876"), vec![])],
    )
    .await;
    add_card(&env, "XXXXXXXXXXXX9876").await;
    let id = env.upload_one("a.pdf", b"a").await;

    let preview = env.get(&format!("/statements/{id}")).await;

    let section = &preview.json["sections"][0];
    assert_eq!(section["matched_without_number"], false);
    assert!(!section["matched_account"].is_null());
    env.finish().await;
}

#[tokio::test]
async fn a_second_statement_dedupes_rows_whose_narration_was_truncated() {
    let mut full = tx(2, "NEFT CR ACME CORP SALARY FOR JUNE 2026", 5000);
    full.balance_paise = Some(9000);
    let mut cut = tx(2, "NEFT CR ACME", 5000);
    cut.balance_paise = Some(9000);
    let env = Env::new(
        "am_truncated",
        vec![
            uni(Some("XX9876"), vec![full]),
            uni(Some("XX9876"), vec![cut]),
        ],
    )
    .await;
    let first = env.upload_one("a.pdf", b"a").await;
    env.confirm_proposed(first, 1).await;
    let second = env.upload_one("b.pdf", b"b").await;

    let preview = env.get(&format!("/statements/{second}")).await;
    let reply = env.confirm_proposed(second, 1).await;

    assert_eq!(preview.json["sections"][0]["duplicate_count"], 1);
    assert_eq!(
        reply.json,
        json!({"added": 0, "duplicates": 1, "linked": 0})
    );
    env.finish().await;
}
