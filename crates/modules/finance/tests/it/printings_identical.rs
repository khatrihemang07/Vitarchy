//! A Statement that prints a Transaction exactly as a kept, live Printing adds no new one
//! (ADR 0015): confirm, preview and Read again agree.

use serde_json::Value;

use crate::common::{
    app::Env,
    pairs::{confirmed, uploaded},
    results::with_balance,
};

fn same() -> vitarchy_finance::reader::contract::Transaction {
    with_balance(5, "UPI/ACME STORE/4455/order", 500, 9000)
}

fn with_reference() -> vitarchy_finance::reader::contract::Transaction {
    let mut row = same();
    row.reference = Some("R-77".into());
    row
}

async fn printings(env: &Env) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM finance.printings")
        .fetch_one(&env.db.pool)
        .await
        .unwrap()
}

async fn flagged(env: &Env) -> bool {
    env.get("/transactions").await.json["items"][0]["no_longer_found"] == true
}

async fn apply(env: &Env, id: i64) -> Value {
    let reply = env
        .post(&format!("/statements/{id}/read-again/confirm"), "{}")
        .await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    reply.json
}

async fn preview(env: &Env, id: i64) -> Value {
    let reply = env
        .post(&format!("/statements/{id}/read-again"), "{}")
        .await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    reply.json
}

#[tokio::test]
async fn the_same_rows_in_two_statements_keep_one_printing_per_transaction() {
    let rows = || vec![same(), with_balance(6, "rent", 700, 8300)];
    let (env, _) = confirmed("pi_overlap", vec![rows(), rows()], vec![]).await;

    assert_eq!(printings(&env).await, 2);
    assert_eq!(env.get("/transactions").await.json["total"], 2);
    env.finish().await;
}

#[tokio::test]
async fn a_second_statement_that_differs_still_adds_a_printing() {
    let (env, _) = confirmed(
        "pi_differs",
        vec![vec![same()], vec![with_reference()]],
        vec![],
    )
    .await;

    assert_eq!(printings(&env).await, 2);
    env.finish().await;
}

#[tokio::test]
async fn read_again_of_an_identical_statement_adds_nothing_until_the_first_printing_is_gone() {
    let (env, ids) = confirmed(
        "pi_again",
        vec![vec![same()], vec![same()]],
        vec![
            vec![same()],
            vec![same()],
            vec![],
            vec![same()],
            vec![same()],
        ],
    )
    .await;
    assert_eq!(printings(&env).await, 1, "the second Statement stored none");

    let shown = preview(&env, ids[1]).await;
    let done = apply(&env, ids[1]).await;
    assert_eq!(shown["sections"][0]["new_printing_count"], 0);
    assert_eq!(done["printings_added"], 0);
    assert_eq!(printings(&env).await, 1);

    // The first Statement no longer finds the row: the Transaction is flagged.
    apply(&env, ids[0]).await;
    assert!(flagged(&env).await, "it only knows its first Statement");

    // The second Statement, read again, now adds its Printing and revives it.
    let revived = apply(&env, ids[1]).await;
    assert_eq!(revived["printings_added"], 1);
    assert_eq!(printings(&env).await, 2);
    assert!(!flagged(&env).await);
    env.finish().await;
}

#[tokio::test]
async fn confirming_an_identical_statement_stores_no_printing() {
    let (env, ids) = uploaded("pi_confirm", vec![vec![same()], vec![same()]], vec![]).await;
    assert_eq!(env.confirm_proposed(ids[0], 1).await.status, 200);

    let reply = env.confirm_proposed(ids[1], 1).await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    assert_eq!(printings(&env).await, 1);
    env.finish().await;
}
