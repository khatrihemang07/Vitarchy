//! Confirms that are refused, and the lock that lets only one confirm run.

use crate::common::{
    app::Env,
    cards::{card, transaction_total},
    results::locked,
    tx,
};
use serde_json::json;
use vitarchy_finance::repo::{confirm_lock, statements};

#[tokio::test]
async fn a_file_that_was_not_read_cannot_be_confirmed() {
    let env = Env::new("cf_locked", vec![locked()]).await;
    let id = env.upload_one("a.pdf", b"a").await;

    let reply = env.confirm_proposed(id, 0).await;

    assert_eq!(
        (reply.status, reply.json["code"].as_str()),
        (409, Some("statement_not_readable"))
    );
    env.finish().await;
}

#[tokio::test]
async fn confirming_twice_is_refused() {
    let env = Env::new("cf_twice", vec![card(vec![tx(2, "A", 100)])]).await;
    let id = env.upload_one("a.pdf", b"a").await;
    env.confirm_proposed(id, 1).await;

    let reply = env.confirm_proposed(id, 1).await;

    assert_eq!(
        (reply.status, reply.json["code"].as_str()),
        (409, Some("statement_confirmed"))
    );
    assert_eq!(transaction_total(&env).await, 1);
    env.finish().await;
}

#[tokio::test]
async fn confirming_a_missing_statement_is_a_404() {
    let env = Env::new("cf_missing", vec![locked()]).await;

    let reply = env.confirm_proposed(999, 0).await;

    assert_eq!(
        (reply.status, reply.json["code"].as_str()),
        (404, Some("statement_not_found"))
    );
    env.finish().await;
}

#[tokio::test]
async fn the_choices_must_fit_the_statement() {
    let env = Env::new("cf_choices", vec![card(vec![tx(2, "A", 100)])]).await;
    let id = env.upload_one("a.pdf", b"a").await;
    let cases = [
        (json!({"sections": []}), 400, "sections_mismatch"),
        (json!({}), 400, "sections_mismatch"),
        (
            json!({"sections": [{"account_id": 1, "new_account": {"issuer": "X", "kind": "loan", "name": "n"}}]}),
            400,
            "invalid_section",
        ),
        (
            json!({"sections": [{"new_account": {"issuer": "X", "kind": "brokerage", "name": "n"}}]}),
            400,
            "invalid_new_account",
        ),
        (
            json!({"sections": [{"new_account": {"issuer": "X", "kind": "loan", "name": "  "}}]}),
            400,
            "invalid_new_account",
        ),
        (
            json!({"sections": [{"account_id": 999}]}),
            404,
            "account_not_found",
        ),
    ];

    for (body, status, code) in cases {
        let reply = env
            .post(&format!("/statements/{id}/confirm"), &body.to_string())
            .await;
        assert_eq!(
            (reply.status, reply.json["code"].as_str()),
            (status, Some(code)),
            "{body}"
        );
    }
    assert_eq!(
        env.get(&format!("/statements/{id}")).await.json["status"],
        "unconfirmed"
    );
    assert_eq!(transaction_total(&env).await, 0);
    env.finish().await;
}

#[tokio::test]
async fn a_running_confirm_makes_another_get_a_409_and_writes_nothing() {
    let env = Env::new("cf_concurrent", vec![card(vec![tx(2, "A", 100)])]).await;
    let id = env.upload_one("a.pdf", b"a").await;
    let mut holder = env.db.pool.begin().await.unwrap();
    assert!(confirm_lock::try_acquire(&mut holder).await.unwrap());

    let blocked = env.confirm_proposed(id, 1).await;
    holder.rollback().await.unwrap();
    let freed = env.confirm_proposed(id, 1).await;

    assert_eq!(
        (blocked.status, blocked.json["code"].as_str()),
        (409, Some("confirm_in_progress"))
    );
    assert_eq!(freed.status, 200, "{}", freed.text);
    let row = statements::get(&env.db.pool, id).await.unwrap().unwrap();
    assert_eq!(row.status, "confirmed");
    assert_eq!(transaction_total(&env).await, 1);
    env.finish().await;
}

#[tokio::test]
async fn each_confirm_is_recorded_as_a_run() {
    let env = Env::new("cf_runs", vec![card(vec![tx(2, "A", 100)])]).await;
    let id = env.upload_one("a.pdf", b"a").await;
    env.confirm_proposed(999, 1).await;
    env.confirm_proposed(id, 1).await;

    let status = env.get("/status").await;

    let run = status.json["latest_runs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["kind"] == "confirm")
        .unwrap()
        .clone();
    assert_eq!(
        (run["outcome"].as_str(), run["counts"]["added"].as_i64()),
        (Some("success"), Some(1))
    );
    assert_eq!(run["statement_id"], id);
    env.finish().await;
}
