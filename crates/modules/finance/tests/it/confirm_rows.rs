//! Confirming a Statement stores its rows once, and only the new ones of an overlapping file.

use crate::common::{
    app::Env,
    cards::{MASK, card, transaction_total},
    results::{card_section, failing, read},
    tx,
};
use serde_json::json;

#[tokio::test]
async fn confirming_creates_the_account_and_stores_every_row() {
    let env = Env::new(
        "cf_basic",
        vec![card(vec![tx(2, "A", 100), tx(3, "B", 200)])],
    )
    .await;
    let id = env.upload_one("a.pdf", b"one").await;

    let reply = env.confirm_proposed(id, 1).await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    assert_eq!(
        reply.json,
        json!({"added": 2, "already_stored": 0, "enriched": 0})
    );
    let accounts = env.get("/accounts").await;
    assert_eq!(accounts.json[0]["issuer"], "ICICI Bank");
    assert_eq!(accounts.json[0]["kind"], "credit_card");
    assert_eq!(accounts.json[0]["name"], "ICICI Bank credit card 2003");
    assert_eq!(accounts.json[0]["transaction_count"], 2);
    let preview = env.get(&format!("/statements/{id}")).await;
    assert_eq!(preview.json["status"], "confirmed");
    assert_eq!(
        preview.json["sections"][0]["matched_account"]["id"],
        accounts.json[0]["id"]
    );
    env.finish().await;
}

#[tokio::test]
async fn an_overlapping_second_statement_adds_only_its_new_rows() {
    let env = Env::new(
        "cf_overlap",
        vec![
            card(vec![tx(1, "A", 100), tx(2, "B", 200), tx(3, "C", 300)]),
            card(vec![tx(2, "B", 200), tx(3, "C", 300), tx(4, "D", 400)]),
        ],
    )
    .await;
    let first = env.upload_one("jun.pdf", b"jun").await;
    env.confirm_proposed(first, 1).await;
    let second = env.upload_one("jul.pdf", b"jul").await;

    let preview = env.get(&format!("/statements/{second}")).await;
    let reply = env.confirm_proposed(second, 1).await;

    let section = &preview.json["sections"][0];
    assert_eq!(
        (
            section["new_count"].as_i64(),
            section["already_stored_count"].as_i64()
        ),
        (Some(1), Some(2))
    );
    assert_eq!(
        reply.json,
        json!({"added": 1, "already_stored": 2, "enriched": 0})
    );
    assert_eq!(transaction_total(&env).await, 4);
    assert_eq!(env.get("/accounts").await.json.as_array().unwrap().len(), 1);
    env.finish().await;
}

#[tokio::test]
async fn two_identical_rows_in_one_file_stay_two() {
    let env = Env::new(
        "cf_twins",
        vec![
            card(vec![tx(2, "COFFEE", 150), tx(2, "COFFEE", 150)]),
            card(vec![
                tx(2, "COFFEE", 150),
                tx(2, "COFFEE", 150),
                tx(2, "COFFEE", 150),
            ]),
        ],
    )
    .await;
    let first = env.upload_one("a.pdf", b"a").await;
    let reply = env.confirm_proposed(first, 1).await;
    assert_eq!(reply.json["added"], 2);
    let second = env.upload_one("b.pdf", b"b").await;

    let reply = env.confirm_proposed(second, 1).await;

    assert_eq!(
        (
            reply.json["added"].as_i64(),
            reply.json["already_stored"].as_i64()
        ),
        (Some(1), Some(2))
    );
    assert_eq!(transaction_total(&env).await, 3);
    env.finish().await;
}

#[tokio::test]
async fn a_statement_with_a_failed_check_can_still_be_confirmed() {
    let env = Env::new(
        "cf_failed_check",
        vec![read(vec![failing(card_section(
            MASK,
            vec![tx(2, "A", 100)],
        ))])],
    )
    .await;
    let id = env.upload_one("a.pdf", b"a").await;

    let preview = env.get(&format!("/statements/{id}")).await;
    let reply = env.confirm_proposed(id, 1).await;

    assert_eq!(
        preview.json["sections"][0]["checks"][0]["outcome"],
        "failed"
    );
    assert_eq!(reply.status, 200);
    assert_eq!(reply.json["added"], 1);
    env.finish().await;
}
