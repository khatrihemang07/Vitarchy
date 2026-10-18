//! Read again on a confirmed Statement: a diff preview, then an approval that marks vanished
//! rows and never deletes.

use crate::common::{
    app::Env,
    results::{card_section, locked, read},
    tx,
};
use serde_json::json;
use vitarchy_finance::reader::contract::Transaction;

fn card(rows: Vec<Transaction>) -> vitarchy_finance::reader::ReaderResult {
    read(vec![card_section("XXXX2003", rows)])
}

fn abc() -> Vec<Transaction> {
    vec![tx(1, "A", 100), tx(2, "B", 200), tx(3, "C", 300)]
}

async fn row_count(env: &Env) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM finance.transactions")
        .fetch_one(&env.db.pool)
        .await
        .unwrap()
}

async fn confirmed(label: &str, later: Vec<vitarchy_finance::reader::ReaderResult>) -> (Env, i64) {
    let mut results = vec![card(abc())];
    results.extend(later);
    let env = Env::new(label, results).await;
    let id = env.upload_one("a.pdf", b"original").await;
    assert_eq!(env.confirm_proposed(id, 1).await.status, 200);
    (env, id)
}

#[tokio::test]
async fn the_preview_counts_additions_and_vanished_rows_and_writes_nothing() {
    let changed = card(vec![tx(1, "A", 100), tx(2, "B", 200), tx(4, "D", 400)]);
    let (env, id) = confirmed("ra_preview", vec![changed]).await;

    let reply = env
        .post(&format!("/statements/{id}/read-again"), "{}")
        .await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    let s = &reply.json["sections"][0];
    assert_eq!(
        (
            s["add_count"].as_i64(),
            s["unchanged_count"].as_i64(),
            s["no_longer_found_count"].as_i64()
        ),
        (Some(1), Some(2), Some(1))
    );
    assert_eq!(s["added"][0]["description"], "D");
    assert_eq!(reply.json["statement"]["status"], "confirmed");
    assert_eq!(row_count(&env).await, 3);
    env.finish().await;
}

#[tokio::test]
async fn approving_adds_new_rows_marks_vanished_ones_and_deletes_nothing() {
    let changed = card(vec![tx(1, "A", 100), tx(2, "B", 200), tx(4, "D", 400)]);
    let (env, id) = confirmed("ra_apply", vec![changed.clone(), changed]).await;
    env.post(&format!("/statements/{id}/read-again"), "{}")
        .await;

    let applied = env
        .post(&format!("/statements/{id}/read-again/confirm"), "{}")
        .await;

    assert_eq!(applied.status, 200, "{}", applied.text);
    assert_eq!(
        applied.json,
        json!({"added": 1, "unchanged": 2, "no_longer_found": 1, "linked": 0})
    );
    assert_eq!(row_count(&env).await, 4);
    let rows = env.get("/transactions").await;
    let lost: Vec<_> = rows.json["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["no_longer_found"] == true)
        .collect();
    assert_eq!(lost.len(), 1);
    assert_eq!(lost[0]["description"], "C");
    let accounts = env.get("/accounts").await;
    assert_eq!(
        accounts.json[0]["transaction_count"], 3,
        "vanished rows are not counted"
    );
    env.finish().await;
}

#[tokio::test]
async fn a_row_found_again_is_revived() {
    let without_c = card(vec![tx(1, "A", 100), tx(2, "B", 200)]);
    let (env, id) = confirmed("ra_revive", vec![without_c, card(abc())]).await;
    env.post(&format!("/statements/{id}/read-again/confirm"), "{}")
        .await;

    let again = env
        .post(&format!("/statements/{id}/read-again/confirm"), "{}")
        .await;

    assert_eq!(
        again.json,
        json!({"added": 0, "unchanged": 3, "no_longer_found": 0, "linked": 0})
    );
    let rows = env.get("/transactions").await;
    assert!(
        rows.json["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["no_longer_found"] == false)
    );
    env.finish().await;
}

#[tokio::test]
async fn rereading_an_unchanged_statement_changes_nothing() {
    let (env, id) = confirmed("ra_same", vec![card(abc())]).await;

    let preview = env
        .post(&format!("/statements/{id}/read-again"), "{}")
        .await;
    let applied = env
        .post(&format!("/statements/{id}/read-again/confirm"), "{}")
        .await;

    let s = &preview.json["sections"][0];
    assert_eq!(
        (s["add_count"].as_i64(), s["no_longer_found_count"].as_i64()),
        (Some(0), Some(0))
    );
    assert_eq!(
        applied.json,
        json!({"added": 0, "unchanged": 3, "no_longer_found": 0, "linked": 0})
    );
    env.finish().await;
}

#[tokio::test]
async fn rows_of_other_statements_are_not_marked_lost() {
    let july_only = card(vec![tx(4, "D", 400)]);
    let (env, _first) = confirmed("ra_others", vec![july_only.clone(), july_only]).await;
    let second = env.upload_one("b.pdf", b"second").await;
    env.confirm_proposed(second, 1).await;

    let applied = env
        .post(&format!("/statements/{second}/read-again/confirm"), "{}")
        .await;

    assert_eq!(applied.json["no_longer_found"], 0, "{}", applied.text);
    env.finish().await;
}

#[tokio::test]
async fn an_engine_that_no_longer_reads_the_file_is_a_409() {
    let (env, id) = confirmed("ra_unreadable", vec![locked()]).await;

    let preview = env
        .post(&format!("/statements/{id}/read-again"), "{}")
        .await;
    let applied = env
        .post(&format!("/statements/{id}/read-again/confirm"), "{}")
        .await;

    for reply in [preview, applied] {
        assert_eq!(
            (reply.status, reply.json["code"].as_str()),
            (409, Some("statement_not_readable"))
        );
    }
    assert_eq!(row_count(&env).await, 3);
    env.finish().await;
}

#[tokio::test]
async fn approving_an_unconfirmed_statement_is_refused() {
    let env = Env::new("ra_unconfirmed", vec![card(abc())]).await;
    let id = env.upload_one("a.pdf", b"x").await;

    let reply = env
        .post(&format!("/statements/{id}/read-again/confirm"), "{}")
        .await;

    assert_eq!(
        (reply.status, reply.json["code"].as_str()),
        (409, Some("statement_unconfirmed"))
    );
    env.finish().await;
}

#[tokio::test]
async fn different_accounts_in_the_file_are_refused() {
    let two = read(vec![
        card_section("XXXX2003", abc()),
        card_section("XXXX2002", abc()),
    ]);
    let (env, id) = confirmed("ra_sections", vec![two]).await;

    let reply = env
        .post(&format!("/statements/{id}/read-again"), "{}")
        .await;

    assert_eq!(
        (reply.status, reply.json["code"].as_str()),
        (409, Some("statement_sections_changed"))
    );
    env.finish().await;
}

#[tokio::test]
async fn read_again_runs_are_recorded() {
    let (env, id) = confirmed("ra_runs", vec![card(abc())]).await;
    env.post(&format!("/statements/{id}/read-again/confirm"), "{}")
        .await;

    let status = env.get("/status").await;

    let run = status.json["latest_runs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["kind"] == "read_again")
        .unwrap()
        .clone();
    assert_eq!(
        (run["outcome"].as_str(), run["counts"]["unchanged"].as_i64()),
        (Some("success"), Some(3))
    );
    env.finish().await;
}
