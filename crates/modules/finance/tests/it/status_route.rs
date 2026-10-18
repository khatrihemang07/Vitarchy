//! The status route.

use crate::common::{
    app::Env,
    results::{card_section, locked, read},
    tx,
};

#[tokio::test]
async fn an_empty_module_reports_zeroes_and_an_available_reader() {
    let env = Env::new("status_empty", vec![locked()]).await;

    let reply = env.get("/status").await;

    assert_eq!(reply.status, 200);
    assert_eq!(reply.json["account_count"], 0);
    assert_eq!(reply.json["unconfirmed_statement_count"], 0);
    assert_eq!(reply.json["unlinked_payment_record_count"], 0);
    assert_eq!(reply.json["reader_available"], true);
    assert!(reply.json["latest_runs"].as_array().unwrap().is_empty());
    env.finish().await;
}

#[tokio::test]
async fn counts_and_the_latest_run_per_kind_follow_the_work() {
    let card = read(vec![card_section("XXXX2003", vec![tx(2, "A", 100)])]);
    let env = Env::new("status_work", vec![card]).await;
    let first = env.upload_one("a.pdf", b"a").await;
    env.upload_one("b.pdf", b"b").await;
    env.confirm_proposed(first, 1).await;

    let reply = env.get("/status").await;

    assert_eq!(reply.json["account_count"], 1);
    assert_eq!(reply.json["unconfirmed_statement_count"], 1);
    let kinds: Vec<_> = reply.json["latest_runs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["kind"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(kinds, ["read", "confirm"]);
    assert_eq!(reply.json["latest_runs"][0]["counts"]["transactions"], 1);
    env.finish().await;
}
