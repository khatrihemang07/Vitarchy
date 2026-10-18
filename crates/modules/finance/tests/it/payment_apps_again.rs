//! A payment app's file seen again, or after a bank's, adds nothing that is already stored.

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
async fn reading_the_app_again_after_an_amount_and_date_join_changes_nothing() {
    let mut bank = tx(5, "UPI/DR/SOME MERCHANT/HDFC", 25_000);
    bank.reference = None;
    let app = app_row(tx(6, "Paid to Cafe Corner", 25_000), (11, 20), None, None);
    let env = confirm_all(
        "pa_reread",
        vec![
            bank_file(vec![bank]),
            app_file(vec![app.clone()]),
            app_file(vec![app]),
        ],
    )
    .await;
    confirm_next(&env, "bank.pdf").await;
    let phone = confirm_next(&env, "phone.csv").await;

    let preview = env
        .post(&format!("/statements/{phone}/read-again"), "{}")
        .await;

    let section = &preview.json["sections"][0];
    assert_eq!(
        (
            section["add_count"].clone(),
            section["new_printing_count"].clone(),
            section["no_longer_found_count"].clone()
        ),
        (json!(0), json!(0), json!(0)),
        "{}",
        preview.text
    );
    env.finish().await;
}

#[tokio::test]
async fn a_second_app_file_repeating_a_payment_does_not_duplicate_it_even_when_the_bank_has_its_own_id()
 {
    let mut bank = bank_row(5);
    bank.source_id = Some("BANK-77".to_owned());
    let app = || {
        app_row(
            tx(5, "Paid to SOME MERCHANT", 25_000),
            (11, 20),
            Some("T1"),
            Some(UTR),
        )
    };
    let env = confirm_all(
        "pa_repeat_file",
        vec![
            bank_file(vec![bank]),
            app_file(vec![app()]),
            app_file(vec![app()]),
        ],
    )
    .await;
    confirm_next(&env, "bank.pdf").await;
    confirm_next(&env, "phone.csv").await;

    confirm_next(&env, "phone-later.csv").await;

    assert_eq!(printings_of_the_only_transaction(&env).await, 2);
    env.finish().await;
}

#[tokio::test]
async fn a_banks_own_row_id_does_not_stop_a_payment_apps_utr_from_matching() {
    let mut bank = bank_row(5);
    bank.source_id = Some("ROW-77".to_owned());
    let app = app_row(
        tx(5, "Paid to SOME MERCHANT", 25_000),
        (11, 20),
        Some("T1"),
        Some(UTR),
    );
    let env = confirm_all("pa_ids", vec![bank_file(vec![bank]), app_file(vec![app])]).await;

    confirm_next(&env, "bank.pdf").await;
    confirm_next(&env, "phone.csv").await;

    assert_eq!(printings_of_the_only_transaction(&env).await, 2);
    env.finish().await;
}
