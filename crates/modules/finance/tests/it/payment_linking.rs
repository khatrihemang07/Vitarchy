//! Payment histories: records link to Transactions by UTR (refunds by direction) and wait
//! for a Statement that does not exist yet.

use crate::common::{
    app::Env,
    payment,
    results::{credit, history, read, section},
    tx,
};
use vitarchy_finance::reader::contract::{AccountKind, Direction, PaymentRecord, Transaction};

const UTR: &str = "512345678901";

fn savings(rows: Vec<Transaction>) -> vitarchy_finance::reader::ReaderResult {
    read(vec![section(
        Some("SBI"),
        Some(AccountKind::Savings),
        "XXXXXX4321",
        rows,
    )])
}

fn upi_rows() -> Vec<Transaction> {
    let mut paid = tx(5, &format!("UPI/DR/{UTR}/SOME MERCHANT"), 25_000);
    paid.reference = Some(UTR.to_owned());
    let mut refund = credit(7, &format!("UPI/CR/{UTR}/SOME MERCHANT"), 25_000);
    refund.reference = Some(UTR.to_owned());
    vec![paid, refund, tx(8, "OTHER SPEND", 25_000)]
}

fn records() -> Vec<PaymentRecord> {
    let mut refund = payment(Some("R2"), Some(UTR), 25_000);
    refund.direction = Direction::Credit;
    refund.occurred_at = refund.occurred_at.date().and_hms_opt(9, 0, 0).unwrap();
    vec![payment(Some("R1"), Some(UTR), 25_000), refund]
}

#[tokio::test]
async fn a_payment_history_links_by_utr_and_its_refund_pair_links_by_direction() {
    let env = Env::new("pl_utr", vec![savings(upi_rows()), history(records())]).await;
    let bank = env.upload_one("bank.pdf", b"bank").await;
    env.confirm_proposed(bank, 1).await;
    let phone = env.upload(&[("phonepe.csv", b"history")]).await;

    let preview = &phone.json[0];
    let confirmed = env
        .confirm_proposed(preview["id"].as_i64().unwrap(), 0)
        .await;

    assert_eq!(preview["kind"], "payment_history");
    assert_eq!(preview["payment_history"]["record_count"], 2);
    assert_eq!(preview["payment_history"]["would_link_now"], 2);
    assert!(preview["sections"].as_array().unwrap().is_empty());
    assert_eq!(
        confirmed.json,
        serde_json::json!({"added": 2, "duplicates": 0, "linked": 2})
    );
    let rows = env.get("/transactions").await;
    let linked: Vec<_> = rows.json["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| !r["payment_record"].is_null())
        .collect();
    assert_eq!(linked.len(), 2);
    let debit = linked.iter().find(|r| r["direction"] == "debit").unwrap();
    let credit = linked.iter().find(|r| r["direction"] == "credit").unwrap();
    assert_eq!(debit["payment_record"]["utr"], UTR);
    assert_eq!(credit["payment_record"]["utr"], UTR);
    assert_ne!(
        debit["payment_record"]["id"],
        credit["payment_record"]["id"]
    );
    assert_eq!(debit["payment_record"]["instrument"], "XXXXXX4321");
    env.finish().await;
}

#[tokio::test]
async fn waiting_records_link_when_a_later_statement_is_confirmed() {
    let env = Env::new("pl_wait", vec![history(records()), savings(upi_rows())]).await;
    let phone = env.upload_one("phonepe.csv", b"history").await;

    let first = env.confirm_proposed(phone, 0).await;
    let waiting = env.get("/status").await;
    let bank = env.upload_one("bank.pdf", b"bank").await;
    let second = env.confirm_proposed(bank, 1).await;
    let after = env.get("/status").await;

    assert_eq!(
        first.json,
        serde_json::json!({"added": 2, "duplicates": 0, "linked": 0})
    );
    assert_eq!(waiting.json["unlinked_payment_record_count"], 2);
    assert_eq!(
        second.json,
        serde_json::json!({"added": 3, "duplicates": 0, "linked": 2})
    );
    assert_eq!(after.json["unlinked_payment_record_count"], 0);
    env.finish().await;
}

#[tokio::test]
async fn records_already_stored_are_not_added_twice() {
    let env = Env::new("pl_dup", vec![history(records()), history(records())]).await;
    let first = env.upload_one("a.csv", b"one").await;
    env.confirm_proposed(first, 0).await;
    let second = env.upload_one("b.csv", b"two").await;

    let reply = env.confirm_proposed(second, 0).await;

    assert_eq!(
        (
            reply.json["added"].as_i64(),
            reply.json["duplicates"].as_i64()
        ),
        (Some(0), Some(2))
    );
    env.finish().await;
}

#[tokio::test]
async fn a_payment_history_takes_no_account_choices() {
    let env = Env::new("pl_choices", vec![history(records())]).await;
    let id = env.upload_one("a.csv", b"one").await;

    let reply = env.confirm_proposed(id, 1).await;

    assert_eq!(
        (reply.status, reply.json["code"].as_str()),
        (400, Some("sections_mismatch"))
    );
    env.finish().await;
}

#[tokio::test]
async fn a_record_from_an_unimported_instrument_stays_unlinked() {
    let mut elsewhere = payment(Some("R9"), None, 25_000);
    elsewhere.instrument = Some("XXXXXX9999".to_owned());
    elsewhere.instrument_digits = Some("9999".to_owned());
    let same_amount_same_day = vec![tx(5, "ANOTHER SHOP", 25_000)];
    let env = Env::new(
        "pl_instrument",
        vec![savings(same_amount_same_day), history(vec![elsewhere])],
    )
    .await;
    let bank = env.upload_one("bank.pdf", b"bank").await;
    env.confirm_proposed(bank, 1).await;
    let phone = env.upload_one("phonepe.csv", b"history").await;

    let preview = env.get(&format!("/statements/{phone}")).await;
    let confirmed = env.confirm_proposed(phone, 0).await;

    assert_eq!(preview.json["payment_history"]["would_link_now"], 0);
    assert_eq!(confirmed.json["linked"], 0);
    assert_eq!(
        env.get("/status").await.json["unlinked_payment_record_count"],
        1
    );
    env.finish().await;
}

#[tokio::test]
async fn a_record_with_no_instrument_links_by_utr_alone() {
    let mut unknown = payment(Some("R8"), None, 25_000);
    unknown.instrument = None;
    unknown.instrument_digits = None;
    let env = Env::new(
        "pl_no_instrument",
        vec![savings(vec![tx(5, "SHOP", 25_000)]), history(vec![unknown])],
    )
    .await;
    let bank = env.upload_one("bank.pdf", b"bank").await;
    env.confirm_proposed(bank, 1).await;
    let phone = env.upload_one("phonepe.csv", b"history").await;

    let confirmed = env.confirm_proposed(phone, 0).await;

    assert_eq!(confirmed.json["linked"], 0);
    env.finish().await;
}
