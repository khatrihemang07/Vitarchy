//! Read all again: one preview per Statement with a stored original, then one approval for
//! those with changes.

use crate::common::{
    app::Env,
    results::{card_section, locked, read},
    tx,
};
use serde_json::json;
use vitarchy_finance::reader::{ReaderResult, contract::Transaction};

fn card(mask: &str, rows: Vec<Transaction>) -> ReaderResult {
    read(vec![card_section(mask, rows)])
}

fn abc() -> Vec<Transaction> {
    vec![tx(1, "A", 100), tx(2, "B", 200), tx(3, "C", 300)]
}

fn xyz() -> Vec<Transaction> {
    vec![tx(4, "X", 400), tx(5, "Y", 500)]
}

/// Two confirmed Statements (first `2003`, second `2002`); `later` answers the reads after.
async fn two_confirmed(label: &str, later: Vec<ReaderResult>) -> (Env, i64, i64) {
    let mut results = vec![card("XXXX2003", abc()), card("XXXX2002", xyz())];
    results.extend(later);
    let env = Env::new(label, results).await;
    let first = env.upload_one("a.pdf", b"first").await;
    let second = env.upload_one("b.pdf", b"second").await;
    assert_eq!(env.confirm_proposed(first, 1).await.status, 200);
    assert_eq!(env.confirm_proposed(second, 1).await.status, 200);
    (env, first, second)
}

fn changed_first() -> ReaderResult {
    card(
        "XXXX2003",
        vec![tx(1, "A", 100), tx(2, "B", 200), tx(4, "D", 400)],
    )
}

async fn row_count(env: &Env) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM finance.transactions")
        .fetch_one(&env.db.pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn the_preview_has_one_item_per_statement_and_writes_nothing() {
    let (env, first, second) = two_confirmed(
        "rall_preview",
        vec![changed_first(), card("XXXX2002", xyz())],
    )
    .await;

    let reply = env.post("/statements/read-again", "{}").await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    let items = reply.json.as_array().unwrap();
    assert_eq!(items.len(), 2);
    let counts = |i: usize| {
        let s = &items[i]["preview"]["sections"][0];
        (
            items[i]["statement_id"].as_i64().unwrap(),
            s["add_count"].as_i64().unwrap(),
            s["unchanged_count"].as_i64().unwrap(),
            s["no_longer_found_count"].as_i64().unwrap(),
        )
    };
    assert_eq!(counts(0), (first, 1, 2, 1));
    assert_eq!(counts(1), (second, 0, 2, 0));
    assert_eq!(row_count(&env).await, 5);
    env.finish().await;
}

#[tokio::test]
async fn approving_applies_only_the_statements_with_changes() {
    let (env, _, _) = two_confirmed(
        "rall_apply",
        vec![
            changed_first(),
            card("XXXX2002", xyz()),
            changed_first(),
            card("XXXX2002", xyz()),
        ],
    )
    .await;
    env.post("/statements/read-again", "{}").await;

    let applied = env.post("/statements/read-again/confirm", "{}").await;

    assert_eq!(applied.status, 200, "{}", applied.text);
    assert_eq!(
        applied.json,
        json!({"statements": 1, "added": 1, "printings_added": 0, "enriched": 0, "no_longer_found": 1, "linked": 0, "failed": 0})
    );
    assert_eq!(row_count(&env).await, 6);
    env.finish().await;
}

#[tokio::test]
async fn a_statement_that_cannot_be_read_again_is_reported_and_skipped() {
    let (env, first, second) = two_confirmed(
        "rall_failure",
        vec![
            locked(),
            card("XXXX2002", xyz()),
            locked(),
            card("XXXX2002", xyz()),
        ],
    )
    .await;

    let reply = env.post("/statements/read-again", "{}").await;
    let applied = env.post("/statements/read-again/confirm", "{}").await;

    let items = reply.json.as_array().unwrap();
    assert_eq!(items[0]["statement_id"], first);
    assert_eq!(items[0]["error_code"], "statement_not_readable");
    assert!(items[0]["preview"].is_null());
    assert_eq!(items[1]["statement_id"], second);
    assert!(items[1]["error_code"].is_null());
    assert_eq!(applied.json["failed"], 1, "{}", applied.text);
    assert_eq!(applied.json["statements"], 0);
    env.finish().await;
}

#[tokio::test]
async fn statements_without_a_stored_original_are_left_out() {
    let (env, first, _) = two_confirmed("rall_missing", vec![card("XXXX2002", xyz())]).await;
    let sha: String =
        sqlx::query_scalar("SELECT file_sha256 FROM finance.statements WHERE id = $1")
            .bind(first)
            .fetch_one(&env.db.pool)
            .await
            .unwrap();
    std::fs::remove_file(env.stored_file(&sha)).unwrap();

    let reply = env.post("/statements/read-again", "{}").await;

    assert_eq!(reply.json.as_array().unwrap().len(), 1, "{}", reply.text);
    env.finish().await;
}

#[tokio::test]
async fn an_unconfirmed_statement_is_read_again_in_place() {
    let env = Env::new("rall_unconfirmed", vec![locked(), card("XXXX2003", abc())]).await;
    let id = env.upload_one("a.pdf", b"x").await;

    let reply = env.post("/statements/read-again", "{}").await;

    let item = &reply.json[0];
    assert_eq!(item["statement_id"], id);
    assert_eq!(item["preview"]["statement"]["status"], "unconfirmed");
    assert_eq!(item["preview"]["statement"]["outcome"], "read");
    env.finish().await;
}

#[tokio::test]
async fn the_request_id_reaches_the_reader() {
    let (env, _, _) = two_confirmed("rall_request_id", vec![card("XXXX2003", abc())]).await;

    env.post_with_request_id("/statements/read-again", "req-all-1")
        .await;
    env.post_with_request_id("/statements/read-again/confirm", "req-all-2")
        .await;

    let ids: Vec<_> = env
        .reader
        .calls()
        .iter()
        .skip(2)
        .map(|c| c.request_id.clone())
        .collect();
    assert!(ids.contains(&Some("req-all-1".to_owned())), "{ids:?}");
    assert!(ids.contains(&Some("req-all-2".to_owned())), "{ids:?}");
    env.finish().await;
}
