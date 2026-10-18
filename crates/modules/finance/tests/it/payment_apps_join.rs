//! A payment app's rows are Printings on the Account that paid: they join the issuer's row of
//! the same payment whichever file comes first, by a shared reference, and stand alone as
//! Transactions when no issuer's row exists.

use serde_json::json;

use crate::common::{
    app_files::{
        UTR, app_file, bank_file, bank_row, confirm_all, confirm_next,
        printings_of_the_only_transaction,
    },
    results::app_row,
    tx,
};

#[tokio::test]
async fn the_app_first_then_the_bank_is_one_transaction_with_two_printings() {
    let app = app_row(
        tx(5, "Paid to SOME MERCHANT", 25_000),
        (11, 20),
        Some("T1"),
        Some(UTR),
    );
    let env = confirm_all(
        "pa_app_first",
        vec![app_file(vec![app]), bank_file(vec![bank_row(5)])],
    )
    .await;

    confirm_next(&env, "phone.csv").await;
    confirm_next(&env, "bank.pdf").await;

    assert_eq!(printings_of_the_only_transaction(&env).await, 2);
    env.finish().await;
}

#[tokio::test]
async fn the_bank_first_then_the_app_is_the_same_result() {
    let app = app_row(
        tx(5, "Paid to SOME MERCHANT", 25_000),
        (11, 20),
        Some("T1"),
        Some(UTR),
    );
    let env = confirm_all(
        "pa_bank_first",
        vec![bank_file(vec![bank_row(5)]), app_file(vec![app])],
    )
    .await;

    confirm_next(&env, "bank.pdf").await;
    confirm_next(&env, "phone.csv").await;

    assert_eq!(printings_of_the_only_transaction(&env).await, 2);
    let shown = &env.get("/transactions").await.json["items"][0];
    assert_eq!(
        shown["description"],
        "UPI/DR/512345678901/SOME MERCHANT/HDFC/xyautopay"
    );
    assert_eq!(
        shown["time"], "11:20:00",
        "the app lends the time the bank does not print"
    );
    env.finish().await;
}

#[tokio::test]
async fn a_matching_utr_joins_rows_a_day_apart() {
    let app = app_row(
        tx(6, "Paid to SOME MERCHANT", 25_000),
        (0, 5),
        Some("T1"),
        Some(UTR),
    );
    let env = confirm_all(
        "pa_utr_day",
        vec![bank_file(vec![bank_row(5)]), app_file(vec![app])],
    )
    .await;

    confirm_next(&env, "bank.pdf").await;
    confirm_next(&env, "phone.csv").await;

    assert_eq!(printings_of_the_only_transaction(&env).await, 2);
    env.finish().await;
}

#[tokio::test]
async fn app_rows_without_a_bank_row_become_transactions_on_their_instruments_account() {
    let app = app_row(
        tx(5, "Paid to SOME MERCHANT", 25_000),
        (11, 20),
        Some("T1"),
        Some(UTR),
    );
    let env = confirm_all("pa_alone", vec![app_file(vec![app])]).await;

    let id = env.upload_one("phone.csv", b"phone").await;
    let confirmed = env.confirm_proposed(id, 1).await;

    assert_eq!(
        confirmed.json,
        json!({"added": 1, "already_stored": 0, "enriched": 0})
    );
    let item = &env.get("/transactions").await.json["items"][0];
    assert_eq!(item["time"], "11:20:00");
    env.finish().await;
}
