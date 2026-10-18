use crate::common::finance_db;
use chrono::{Duration, Utc};
use serde_json::json;
use vitarchy_finance::repo::{
    confirm_lock,
    runs::{self, RunRow},
};

fn run(kind: &str, outcome: &str, offset_secs: i64) -> RunRow {
    let started = Utc::now() + Duration::seconds(offset_secs);
    RunRow {
        kind: kind.to_owned(),
        started_at: started,
        finished_at: started,
        outcome: outcome.to_owned(),
        counts: json!({"added": 3}),
        error_code: (outcome == "failed").then(|| "reader_timeout".to_owned()),
        statement_id: Some(7),
    }
}

#[tokio::test]
async fn latest_returns_the_newest_run_of_a_kind_failed_or_not() {
    let db = finance_db("runs_latest").await;
    runs::record(&db.pool, &run("read", "success", 0))
        .await
        .unwrap();
    runs::record(&db.pool, &run("read", "failed", 10))
        .await
        .unwrap();
    runs::record(&db.pool, &run("confirm", "success", 20))
        .await
        .unwrap();
    let latest = runs::latest(&db.pool, "read").await.unwrap().unwrap();
    assert_eq!(
        (latest.outcome.as_str(), latest.error_code.as_deref()),
        ("failed", Some("reader_timeout"))
    );
    assert_eq!(latest.statement_id, Some(7));
    assert!(
        runs::latest(&db.pool, "read_again")
            .await
            .unwrap()
            .is_none()
    );
    db.drop().await;
}

#[tokio::test]
async fn an_unknown_run_kind_is_refused() {
    let db = finance_db("runs_kind").await;
    assert!(
        runs::record(&db.pool, &run("mystery", "success", 0))
            .await
            .is_err()
    );
    db.drop().await;
}

#[tokio::test]
async fn only_one_confirm_holds_the_lock_and_commit_releases_it() {
    let db = finance_db("confirm_lock").await;
    let mut first = db.pool.begin().await.unwrap();
    let mut second = db.pool.begin().await.unwrap();
    assert!(confirm_lock::try_acquire(&mut first).await.unwrap());
    assert!(!confirm_lock::try_acquire(&mut second).await.unwrap());
    first.commit().await.unwrap();
    assert!(confirm_lock::try_acquire(&mut second).await.unwrap());
    second.rollback().await.unwrap();
    db.drop().await;
}
