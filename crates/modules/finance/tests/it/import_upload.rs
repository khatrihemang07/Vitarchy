//! Uploading Statement files: the unconfirmed preview each one becomes.

use crate::common::{
    account,
    app::Env,
    results::{card_section, credit, locked, read},
    tx,
};
use vitarchy_finance::{
    reader::{ReaderError, testing::FakeReader},
    repo::{account_identifiers, transactions},
};

const MASK: &str = "4123XXXXXXXX2003";

fn one_card(
    rows: Vec<vitarchy_finance::reader::contract::Transaction>,
) -> vitarchy_finance::reader::ReaderResult {
    read(vec![card_section(MASK, rows)])
}

#[tokio::test]
async fn an_upload_becomes_an_unconfirmed_preview_with_totals_and_checks() {
    let env = Env::new(
        "up_preview",
        vec![one_card(vec![
            tx(2, "SHOP ONE", 550_000),
            tx(3, "SHOP TWO", 1_000),
            credit(5, "PAYMENT RECEIVED", 100_000),
        ])],
    )
    .await;

    let reply = env.upload(&[("card.pdf", b"file-one")]).await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    let p = &reply.json[0];
    assert_eq!(p["file_name"], "card.pdf");
    assert_eq!(
        (p["outcome"].as_str(), p["status"].as_str()),
        (Some("read"), Some("unconfirmed"))
    );
    assert_eq!(
        (p["format"].as_str(), p["kind"].as_str()),
        (Some("pdf"), Some("statement"))
    );
    assert_eq!(p["duplicate_file"], false);
    assert_eq!(p["encrypted"], false);
    assert!(p["unlocked_with_label"].is_null());
    assert_eq!(p["period"]["start"], "2026-07-01");
    let s = &p["sections"][0];
    assert_eq!(s["proposed"]["issuer"], "ICICI Bank");
    assert_eq!(s["proposed"]["kind"], "credit_card");
    assert_eq!(s["proposed"]["mask"], MASK);
    assert!(s["matched_account"].is_null());
    assert_eq!(s["transaction_count"], 3);
    assert_eq!(
        (
            s["total_debit_paise"].as_i64(),
            s["total_credit_paise"].as_i64()
        ),
        (Some(551_000), Some(100_000))
    );
    assert_eq!(
        (s["new_count"].as_i64(), s["duplicate_count"].as_i64()),
        (Some(3), Some(0))
    );
    assert_eq!(s["checks"][0]["name"], "balance_chain");
    assert_eq!(s["transactions"][0]["description"], "SHOP ONE");
    assert_eq!(s["transactions"][0]["is_duplicate"], false);
    assert_eq!(s["transactions"][2]["direction"], "credit");
    env.finish().await;
}

#[tokio::test]
async fn uploading_the_same_file_again_returns_the_stored_statement_without_reading() {
    let env = Env::new("up_dup", vec![one_card(vec![tx(2, "SHOP", 100)])]).await;
    let first = env.upload_one("a.pdf", b"same bytes").await;

    let again = env.upload(&[("renamed.pdf", b"same bytes")]).await;

    assert_eq!(again.json[0]["id"], first);
    assert_eq!(again.json[0]["duplicate_file"], true);
    assert_eq!(again.json[0]["file_name"], "a.pdf");
    assert_eq!(env.reader.calls().len(), 1);
    env.finish().await;
}

#[tokio::test]
async fn several_files_give_one_preview_each_in_order() {
    let env = Env::new(
        "up_many",
        vec![
            one_card(vec![tx(2, "A", 100)]),
            one_card(vec![tx(3, "B", 200)]),
        ],
    )
    .await;

    let reply = env.upload(&[("a.pdf", b"aaa"), ("b.pdf", b"bbb")]).await;

    assert_eq!(reply.json.as_array().unwrap().len(), 2);
    assert_eq!(reply.json[0]["file_name"], "a.pdf");
    assert_eq!(
        reply.json[1]["sections"][0]["transactions"][0]["description"],
        "B"
    );
    env.finish().await;
}

#[tokio::test]
async fn an_upload_without_files_is_refused() {
    let env = Env::new("up_none", vec![locked()]).await;

    let reply = env.upload(&[]).await;

    assert_eq!(
        (reply.status, reply.json["code"].as_str()),
        (400, Some("no_files"))
    );
    env.finish().await;
}

#[tokio::test]
async fn a_locked_file_is_kept_so_it_can_be_read_again() {
    let env = Env::new("up_locked", vec![locked()]).await;

    let reply = env.upload(&[("locked.pdf", b"locked")]).await;

    let p = &reply.json[0];
    assert_eq!(p["outcome"], "locked");
    assert_eq!(p["diagnostics"]["stage"], "open");
    assert_eq!(p["diagnostics"]["code"], "no_password_worked");
    assert!(p["sections"].as_array().unwrap().is_empty());
    assert!(
        env.stored_file(&vitarchy_finance::files::sha256_hex(b"locked"))
            .exists()
    );
    env.finish().await;
}

#[tokio::test]
async fn a_crashed_reader_leaves_a_crashed_statement_and_a_failed_run() {
    let env = Env::with_reader(
        "up_crash",
        FakeReader::with_answers([Err(ReaderError::Crashed(Some(1)))]),
    )
    .await;

    let reply = env.upload(&[("a.pdf", b"boom")]).await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    assert_eq!(reply.json[0]["outcome"], "crashed");
    let status = env.get("/status").await;
    assert_eq!(status.json["latest_runs"][0]["kind"], "read");
    assert_eq!(status.json["latest_runs"][0]["outcome"], "failed");
    assert_eq!(
        status.json["latest_runs"][0]["error_code"],
        "reader_crashed"
    );
    env.finish().await;
}

#[tokio::test]
async fn the_preview_matches_a_known_account_and_counts_stored_rows_as_duplicates() {
    let env = Env::new(
        "up_match",
        vec![one_card(vec![tx(2, "SHOP", 100), tx(3, "NEW", 200)])],
    )
    .await;
    let known = account(&env.db, MASK).await;
    transactions::insert(
        &env.db.pool,
        known.id,
        crate::common::statement(&env.db, 'f').await,
        &tx(2, "SHOP", 100),
    )
    .await
    .unwrap();

    let reply = env.upload(&[("a.pdf", b"x")]).await;

    let s = &reply.json[0]["sections"][0];
    assert_eq!(s["matched_account"]["id"], known.id);
    assert_eq!(s["matched_account"]["name"], "Card");
    assert_eq!(
        (s["new_count"].as_i64(), s["duplicate_count"].as_i64()),
        (Some(1), Some(1))
    );
    assert_eq!(s["transactions"][0]["is_duplicate"], true);
    assert_eq!(s["transactions"][1]["is_duplicate"], false);
    env.finish().await;
}

#[tokio::test]
async fn a_remembered_identifier_matches_too() {
    let env = Env::new("up_ident", vec![one_card(vec![tx(2, "SHOP", 100)])]).await;
    let known = account(&env.db, "XXXX9999").await;
    account_identifiers::remember(
        &env.db.pool,
        known.id,
        "ICICI Bank",
        vitarchy_finance::reader::contract::AccountKind::CreditCard,
        "2003",
    )
    .await
    .unwrap();

    let reply = env.upload(&[("a.pdf", b"x")]).await;

    assert_eq!(
        reply.json[0]["sections"][0]["matched_account"]["id"],
        known.id
    );
    env.finish().await;
}
