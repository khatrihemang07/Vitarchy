//! Read again with Printings: gone marks per Statement, matching against every Transaction of
//! the Account, and the Transaction flagged only when no Statement still prints it.

use serde_json::{Value, json};

use crate::common::{app::Env, pairs::confirmed, results::with_balance};

const CUT: &str = "UPI/ACME STORE/4455/payment for the order";
const FULL: &str = "UPI/ACME STORE/4455/payment for the order of nine items";

fn a_cut() -> vitarchy_finance::reader::contract::Transaction {
    with_balance(5, CUT, 500, 9000)
}

fn a_full() -> vitarchy_finance::reader::contract::Transaction {
    with_balance(5, FULL, 500, 9000)
}

fn b() -> vitarchy_finance::reader::contract::Transaction {
    with_balance(6, "rent for july", 700, 8300)
}

async fn row(env: &Env, description_starts: &str) -> Value {
    env.get("/transactions").await.json["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| {
            r["description"]
                .as_str()
                .unwrap()
                .starts_with(description_starts)
        })
        .unwrap()
        .clone()
}

async fn apply(env: &Env, id: i64) -> Value {
    let reply = env
        .post(&format!("/statements/{id}/read-again/confirm"), "{}")
        .await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    reply.json
}

#[tokio::test]
async fn a_vanished_printing_is_marked_and_the_transaction_only_when_every_printing_is_gone() {
    let (env, ids) = confirmed(
        "pra_gone",
        vec![vec![a_cut(), b()], vec![a_full()]],
        vec![vec![b()], vec![]],
    )
    .await;

    let first = apply(&env, ids[0]).await;
    let after_first = row(&env, "UPI").await;
    let second = apply(&env, ids[1]).await;
    let after_second = row(&env, "UPI").await;

    assert_eq!(first["no_longer_found"], 1);
    assert_eq!(
        after_first["no_longer_found"], false,
        "the other Statement still prints it"
    );
    assert_eq!(after_first["description"], FULL);
    assert_eq!(second["no_longer_found"], 1);
    assert_eq!(after_second["no_longer_found"], true);
    assert_eq!(after_second["printing_count"], 2);
    env.finish().await;
}

#[tokio::test]
async fn a_printing_found_again_clears_the_mark() {
    let (env, ids) = confirmed(
        "pra_revive",
        vec![vec![a_cut(), b()]],
        vec![vec![b()], vec![a_cut(), b()]],
    )
    .await;
    apply(&env, ids[0]).await;
    assert_eq!(row(&env, "UPI").await["no_longer_found"], true);

    let again = apply(&env, ids[0]).await;

    assert_eq!(again["unchanged"], 2);
    assert_eq!(row(&env, "UPI").await["no_longer_found"], false);
    env.finish().await;
}

#[tokio::test]
async fn a_row_another_statement_stored_gains_a_printing_instead_of_a_second_transaction() {
    let (env, ids) = confirmed(
        "pra_enrich",
        vec![vec![a_cut()], vec![b()]],
        vec![vec![b(), a_full()]],
    )
    .await;

    let applied = apply(&env, ids[1]).await;

    assert_eq!(
        applied,
        json!({"added": 0, "unchanged": 1, "printings_added": 1, "enriched": 1, "no_longer_found": 0, "linked": 0})
    );
    let shared = row(&env, "UPI").await;
    assert_eq!(
        (
            shared["printing_count"].as_i64(),
            shared["description"].as_str()
        ),
        (Some(2), Some(FULL))
    );
    assert_eq!(env.get("/transactions").await.json["total"], 2);
    env.finish().await;
}

#[tokio::test]
async fn the_preview_reports_printings_that_would_be_added_and_gain_detail() {
    let (env, ids) = confirmed(
        "pra_preview",
        vec![vec![a_cut()], vec![b()]],
        vec![vec![b(), a_full()]],
    )
    .await;

    let reply = env
        .post(&format!("/statements/{}/read-again", ids[1]), "{}")
        .await;

    let section = &reply.json["sections"][0];
    assert_eq!(
        (
            section["add_count"].as_i64(),
            section["new_printing_count"].as_i64(),
            section["enriched_count"].as_i64(),
            section["no_longer_found_count"].as_i64()
        ),
        (Some(0), Some(1), Some(1), Some(0))
    );
    env.finish().await;
}

#[tokio::test]
async fn the_preview_lists_rows_that_would_gain_a_printing_and_tags_the_ones_adding_detail() {
    let (env, ids) = confirmed(
        "pra_rows",
        vec![vec![a_cut()], vec![b()]],
        vec![vec![b(), a_full()]],
    )
    .await;

    let reply = env
        .post(&format!("/statements/{}/read-again", ids[1]), "{}")
        .await;

    let rows = reply.json["sections"][0]["new_printings"]
        .as_array()
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(
        (
            rows[0]["description"].as_str(),
            rows[0]["is_already_stored"].as_bool(),
            rows[0]["gains_detail"].as_bool()
        ),
        (Some(FULL), Some(true), Some(true))
    );
    env.finish().await;
}

#[tokio::test]
async fn read_all_again_sums_the_printings_it_added() {
    let (env, _) = confirmed(
        "pra_all",
        vec![vec![a_cut()], vec![b()]],
        vec![vec![a_cut()], vec![b(), a_full()]],
    )
    .await;

    let applied = env.post("/statements/read-again/confirm", "{}").await;

    assert_eq!(applied.status, 200, "{}", applied.text);
    assert_eq!(
        (
            applied.json["statements"].as_i64(),
            applied.json["printings_added"].as_i64(),
            applied.json["enriched"].as_i64()
        ),
        (Some(1), Some(1), Some(1))
    );
    env.finish().await;
}
