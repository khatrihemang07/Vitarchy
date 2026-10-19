//! What the database keeps: Transfers follow their Transactions, the backfill runs once, and a
//! merge of duplicate Accounts keeps them.

use crate::common::{
    account,
    app::Env,
    cards::{MASK, card},
    results::credit,
    transfers::{import, savings_file, transfer_count, transfer_of},
    tx,
};
use serde_json::json;
use vitarchy_finance::{BACKFILL_MARKER, backfill_once, merge_duplicate_accounts};

async fn linked(label: &str) -> Env {
    let env = Env::new(
        label,
        vec![
            savings_file("XXXX1111", vec![tx(6, "money out", 777_700)]),
            savings_file("XXXX3333", vec![credit(6, "money in", 777_700)]),
        ],
    )
    .await;
    import(&env, "one").await;
    import(&env, "two").await;
    assert_eq!(transfer_count(&env).await, 1);
    env
}

async fn run(env: &Env, sql: &str) {
    sqlx::query(sqlx::AssertSqlSafe(sql.to_owned()))
        .execute(&env.db.pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn deleting_the_debit_dissolves_the_transfer_and_frees_the_credit() {
    let env = linked("trs_debit").await;
    run(&env, "DELETE FROM finance.printings WHERE transaction_id IN (SELECT id FROM finance.transactions WHERE description = 'money out')").await;
    run(
        &env,
        "DELETE FROM finance.transactions WHERE description = 'money out'",
    )
    .await;
    assert_eq!(transfer_count(&env).await, 0);
    assert!(transfer_of(&env, "money in").await.is_null());
    env.finish().await;
}

#[tokio::test]
async fn deleting_the_only_credit_dissolves_the_transfer() {
    let env = linked("trs_credit").await;
    run(&env, "DELETE FROM finance.printings WHERE transaction_id IN (SELECT id FROM finance.transactions WHERE description = 'money in')").await;
    run(
        &env,
        "DELETE FROM finance.transactions WHERE description = 'money in'",
    )
    .await;
    assert_eq!(transfer_count(&env).await, 0);
    assert!(transfer_of(&env, "money out").await.is_null());
    env.finish().await;
}

#[tokio::test]
async fn the_backfill_links_what_was_stored_before_and_runs_once() {
    let env = linked("trs_backfill").await;
    run(&env, "DELETE FROM finance.transfers").await;
    run(&env, "DELETE FROM finance.markers").await;

    assert!(
        backfill_once(&env.db.pool).await.unwrap(),
        "first start runs it"
    );
    assert_eq!(transfer_count(&env).await, 1);
    let marker: i64 = sqlx::query_scalar("SELECT count(*) FROM finance.markers WHERE name = $1")
        .bind(BACKFILL_MARKER)
        .fetch_one(&env.db.pool)
        .await
        .unwrap();
    assert_eq!(marker, 1);

    run(&env, "DELETE FROM finance.transfers").await;
    assert!(
        !backfill_once(&env.db.pool).await.unwrap(),
        "a later start does not"
    );
    assert_eq!(
        transfer_count(&env).await,
        0,
        "so an unlink made since is not undone"
    );
    env.finish().await;
}

#[tokio::test]
async fn the_backfill_respects_remembered_unlinks() {
    let env = linked("trs_backfill_rej").await;
    let id = transfer_of(&env, "money out").await["id"].as_i64().unwrap();
    env.json_call("DELETE", &format!("/transfers/{id}"), None)
        .await;
    run(&env, "DELETE FROM finance.markers").await;

    backfill_once(&env.db.pool).await.unwrap();

    assert_eq!(transfer_count(&env).await, 0);
    env.finish().await;
}

#[tokio::test]
async fn merging_duplicate_accounts_keeps_the_transfer_on_the_surviving_transaction() {
    // Two copies of one card Account print the same credit; the savings debit paid it.
    let env = Env::new(
        "trs_merge",
        vec![
            savings_file("XXXX1111", vec![tx(6, "money out", 777_700)]),
            card(vec![credit(6, "money in", 777_700)]),
            card(vec![credit(6, "money in", 777_700)]),
        ],
    )
    .await;
    import(&env, "bank").await;
    let (first, second) = (
        account(&env.db, MASK).await.id,
        account(&env.db, MASK).await.id,
    );
    for (name, account_id) in [("card a", first), ("card b", second)] {
        let id = env
            .upload_one(&format!("{name}.pdf"), name.as_bytes())
            .await;
        let body = json!({"sections": [{"account_id": account_id}]});
        assert_eq!(
            env.post(&format!("/statements/{id}/confirm"), &body.to_string())
                .await
                .status,
            200
        );
    }
    // The duplicate (second) copy of the credit is the one linked.
    run(&env, "DELETE FROM finance.transfers").await;
    run(&env, &format!(
        "WITH t AS (INSERT INTO finance.transfers (debit_id, rung) SELECT id, 'manual' FROM finance.transactions WHERE description = 'money out' RETURNING id)
         INSERT INTO finance.transfer_credits (transfer_id, credit_id)
         SELECT t.id, x.id FROM t, finance.transactions x WHERE x.description = 'money in' AND x.account_id = {second}")).await;

    merge_duplicate_accounts(&env.db.pool).await.unwrap();

    let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM finance.transactions")
        .fetch_one(&env.db.pool)
        .await
        .unwrap();
    assert_eq!(rows, 2, "the copies folded into one Transaction");
    assert_eq!(transfer_count(&env).await, 1);
    let back = transfer_of(&env, "money in").await;
    assert!(!back.is_null(), "the surviving credit is in the Transfer");
    env.finish().await;
}

#[tokio::test]
async fn the_modules_startup_job_is_the_backfill() {
    let env = linked("trs_startup").await;
    run(&env, "DELETE FROM finance.transfers").await;
    run(&env, "DELETE FROM finance.markers").await;

    let task = vitarchy_finance::module().startup().expect("a startup job");
    task(env.db.pool.clone()).await;

    assert_eq!(transfer_count(&env).await, 1);
    env.finish().await;
}
