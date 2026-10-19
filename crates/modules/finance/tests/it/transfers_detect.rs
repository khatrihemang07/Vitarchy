//! Statements that confirm or are read again link Transfers on their own.

use crate::common::{
    app::Env,
    results::credit,
    transfers::{card_file, import, loan_file, savings_file, transfer_count, transfer_of},
    tx,
};
use vitarchy_core::log_capture::capture_logs;

const PAYEE: &str = "UPI/pay@cred.club/CRED Club bill";

#[tokio::test]
async fn a_card_bill_payment_links_to_its_receipt_when_the_card_statement_arrives() {
    let env = Env::new(
        "tr_bill",
        vec![
            savings_file("XXXX1111", vec![tx(5, PAYEE, 100_000)]),
            card_file(
                "XXXX2222",
                vec![credit(5, "BBPS Payment received", 100_400)],
            ),
        ],
    )
    .await;
    import(&env, "bank").await;
    assert_eq!(
        transfer_count(&env).await,
        0,
        "a lone debit is not a Transfer"
    );

    import(&env, "card").await;

    let out = transfer_of(&env, PAYEE).await;
    let back = transfer_of(&env, "BBPS Payment received").await;
    assert_eq!(out["rung"], "bill_payment");
    assert_eq!(out["id"], back["id"]);
    assert_eq!(out["extra_paise"], 400);
    assert_eq!(out["partial"], false);
    assert_eq!(out["other_accounts"].as_array().unwrap().len(), 1);
    assert_eq!(back["other_accounts"].as_array().unwrap().len(), 1);
    assert_ne!(out["other_accounts"], back["other_accounts"]);
    env.finish().await;
}

#[tokio::test]
async fn a_debit_whose_reference_is_in_the_credit_links_by_reference() {
    let mut out = tx(8, "NEFT to own account", 4_000_000);
    out.reference = Some("XY12345678".into());
    let env = Env::new(
        "tr_ref",
        vec![
            savings_file("XXXX1111", vec![out]),
            savings_file(
                "XXXX3333",
                vec![credit(8, "NEFT-XY12345678-SOMEONE", 4_000_000)],
            ),
        ],
    )
    .await;
    import(&env, "one").await;
    import(&env, "two").await;

    assert_eq!(
        transfer_of(&env, "NEFT to own account").await["rung"],
        "reference"
    );
    env.finish().await;
}

#[tokio::test]
async fn a_debit_naming_a_loan_account_links_to_the_loan_credit() {
    let env = Env::new(
        "tr_acct",
        vec![
            savings_file(
                "XXXX1111",
                vec![
                    tx(4, "DIRECT DR 0000000007777", 500_000),
                    tx(4, "SIP instalment", 500_000),
                ],
            ),
            loan_file("XXXX7777", vec![credit(4, "DEPOSIT TRANSFER", 500_000)]),
        ],
    )
    .await;
    import(&env, "bank").await;
    import(&env, "loan").await;

    assert_eq!(
        transfer_of(&env, "DIRECT DR 0000000007777").await["rung"],
        "account_number"
    );
    assert!(transfer_of(&env, "SIP instalment").await.is_null());
    env.finish().await;
}

#[tokio::test]
async fn a_payment_split_across_cards_is_partial_until_the_other_card_arrives() {
    let env = Env::new(
        "tr_partial",
        vec![
            savings_file("XXXX1111", vec![tx(2, PAYEE, 2_881_100)]),
            card_file("XXXX2222", vec![credit(2, "PAYMENT RECEIVED 11", 574_700)]),
            card_file("XXXX4444", vec![credit(3, "BBPS-PAYMENT", 2_306_400)]),
        ],
    )
    .await;
    import(&env, "bank").await;
    import(&env, "first card").await;

    let partial = transfer_of(&env, PAYEE).await;
    assert_eq!(partial["partial"], true);
    assert_eq!(partial["remaining_paise"], 2_306_400);

    import(&env, "second card").await;

    let done = transfer_of(&env, PAYEE).await;
    assert_eq!(done["id"], partial["id"], "the same Transfer grew");
    assert_eq!(
        (done["partial"].clone(), done["remaining_paise"].clone()),
        (false.into(), 0.into())
    );
    assert_eq!(done["other_accounts"].as_array().unwrap().len(), 2);
    env.finish().await;
}

#[tokio::test]
async fn a_lone_matching_amount_links_when_nothing_stronger_applies() {
    let env = Env::new(
        "tr_unique",
        vec![
            savings_file("XXXX1111", vec![tx(6, "moved to the other bank", 777_700)]),
            savings_file("XXXX3333", vec![credit(7, "money received", 777_700)]),
        ],
    )
    .await;
    import(&env, "one").await;
    import(&env, "two").await;

    assert_eq!(
        transfer_of(&env, "moved to the other bank").await["rung"],
        "unique_amount"
    );
    env.finish().await;
}

#[tokio::test]
async fn an_ambiguous_amount_is_left_unlinked() {
    let env = Env::new(
        "tr_ambig",
        vec![
            savings_file(
                "XXXX1111",
                vec![tx(6, "pay one", 5_000), tx(6, "pay two", 5_000)],
            ),
            savings_file("XXXX3333", vec![credit(6, "received", 5_000)]),
        ],
    )
    .await;
    import(&env, "one").await;
    import(&env, "two").await;

    assert_eq!(transfer_count(&env).await, 0);
    env.finish().await;
}

#[tokio::test]
async fn a_run_logs_the_trigger_and_counts_but_no_description_or_amount() {
    let env = Env::new(
        "tr_log",
        vec![
            savings_file(
                "XXXX1111",
                vec![tx(6, "SECRETDESCRIPTION out", 123_457_700)],
            ),
            savings_file(
                "XXXX3333",
                vec![credit(6, "SECRETDESCRIPTION in", 123_457_700)],
            ),
        ],
    )
    .await;
    import(&env, "one").await;

    let ((), logs) = capture_logs(async {
        import(&env, "two").await;
    })
    .await;

    let line = logs
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .find(|l| l["fields"]["message"] == "transfers detected")
        .unwrap_or_else(|| panic!("no detection line in {logs}"));
    assert_eq!(line["fields"]["trigger"], "confirm");
    assert_eq!(line["fields"]["unique_amount"], 1);
    assert_eq!(line["fields"]["ambiguous"], 0);
    assert!(
        !logs.contains("SECRETDESCRIPTION") && !logs.contains("123457700"),
        "{logs}"
    );
    env.finish().await;
}

#[tokio::test]
async fn detection_logs_carry_the_request_id_of_the_request_that_caused_them() {
    let env = Env::new(
        "tr_reqid",
        vec![
            savings_file("XXXX1111", vec![tx(6, "out", 777_700)]),
            savings_file("XXXX3333", vec![credit(6, "in", 777_700)]),
        ],
    )
    .await;
    import(&env, "one").await;
    let id = env.upload_one("two.pdf", b"two").await;
    let body = serde_json::json!({"sections": [{}]}).to_string();

    let (reply, logs) = capture_logs(async {
        env.call_with_request_id(
            "POST",
            &format!("/statements/{id}/confirm"),
            &body,
            "req-confirm-1",
        )
        .await
    })
    .await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    let line = logs
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .find(|l| l["fields"]["message"] == "transfers detected")
        .unwrap_or_else(|| panic!("no detection line in {logs}"));
    assert_eq!(line["fields"]["request_id"], "req-confirm-1");
    env.finish().await;
}
