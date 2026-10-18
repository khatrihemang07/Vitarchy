//! The last rung: a payment app's row joins an issuer's row of the same amount and date only
//! when each is the other's single candidate and no reference disagrees.

use crate::common::{
    app_files::{
        UTR, app_file, bank_file, confirm_all, confirm_next, printings_of_the_only_transaction,
    },
    cards::transaction_total,
    results::app_row,
    tx,
};

#[tokio::test]
async fn a_lone_amount_and_date_pair_joins_without_any_shared_id() {
    let mut bank = tx(5, "UPI/DR/SOME MERCHANT/HDFC", 25_000);
    bank.reference = None;
    let app = app_row(
        tx(6, "Paid to Cafe Corner", 25_000),
        (11, 20),
        Some("T1"),
        None,
    );
    let env = confirm_all(
        "pa_unique",
        vec![bank_file(vec![bank]), app_file(vec![app])],
    )
    .await;

    confirm_next(&env, "bank.pdf").await;
    confirm_next(&env, "phone.csv").await;

    assert_eq!(printings_of_the_only_transaction(&env).await, 2);
    env.finish().await;
}

#[tokio::test]
async fn two_same_amount_payments_in_the_window_do_not_merge() {
    let bank = [
        tx(5, "UPI/DR/GROCER/HDFC", 25_000),
        tx(5, "NEFT/CR/RENT/SBIN", 25_000),
    ];
    let app = app_row(
        tx(5, "Paid to Cafe Corner", 25_000),
        (11, 20),
        Some("T1"),
        None,
    );
    let env = confirm_all(
        "pa_ambiguous",
        vec![bank_file(bank.to_vec()), app_file(vec![app])],
    )
    .await;

    confirm_next(&env, "bank.pdf").await;
    confirm_next(&env, "phone.csv").await;

    assert_eq!(
        transaction_total(&env).await,
        3,
        "refused, so the app's row stands alone"
    );
    env.finish().await;
}

#[tokio::test]
async fn rows_whose_references_disagree_do_not_join_by_amount_and_date() {
    let mut bank = tx(5, "UPI/DR/SOME MERCHANT/HDFC", 25_000);
    bank.reference = Some("999999999999".to_owned());
    let app = app_row(
        tx(5, "Paid to Cafe Corner", 25_000),
        (11, 20),
        Some("T1"),
        Some(UTR),
    );
    let env = confirm_all(
        "pa_ref_disagree",
        vec![bank_file(vec![bank]), app_file(vec![app])],
    )
    .await;

    confirm_next(&env, "bank.pdf").await;
    confirm_next(&env, "phone.csv").await;

    assert_eq!(
        transaction_total(&env).await,
        2,
        "two different references are two payments"
    );
    env.finish().await;
}
