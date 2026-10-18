//! The Accounts list and renaming.

use crate::common::{
    app::Env,
    results::{card_section, read, with_balance},
    tx,
};
use serde_json::json;

fn two_days() -> vitarchy_finance::reader::ReaderResult {
    read(vec![card_section(
        "XXXX2003",
        vec![
            with_balance(9, "LATEST WITH BALANCE", 100, 7_000),
            tx(12, "NEWEST NO BALANCE", 100),
            with_balance(3, "EARLIER", 100, 9_000),
        ],
    )])
}

#[tokio::test]
async fn the_list_shows_balance_counts_dates_and_statements() {
    let env = Env::new("ac_list", vec![two_days()]).await;
    let id = env.upload_one("a.pdf", b"a").await;
    env.confirm_proposed(id, 1).await;

    let reply = env.get("/accounts").await;

    let a = &reply.json[0];
    assert_eq!(
        a["latest_balance_paise"], 7_000,
        "the latest-dated row that printed a balance"
    );
    assert_eq!(a["transaction_count"], 3);
    assert_eq!(
        (a["first_date"].as_str(), a["last_date"].as_str()),
        (Some("2026-07-03"), Some("2026-07-12"))
    );
    assert_eq!(a["statement_count"], 1);
    assert_eq!(a["mask"], "XXXX2003");
    env.finish().await;
}

#[tokio::test]
async fn an_unconfirmed_statement_creates_no_account() {
    let env = Env::new("ac_none", vec![two_days()]).await;
    env.upload_one("a.pdf", b"a").await;

    assert!(
        env.get("/accounts")
            .await
            .json
            .as_array()
            .unwrap()
            .is_empty()
    );
    env.finish().await;
}

#[tokio::test]
async fn an_account_can_be_renamed() {
    let env = Env::new("ac_rename", vec![two_days()]).await;
    let id = env.upload_one("a.pdf", b"a").await;
    env.confirm_proposed(id, 1).await;
    let account_id = env.get("/accounts").await.json[0]["id"].as_i64().unwrap();

    let reply = env
        .json_call(
            "PATCH",
            &format!("/accounts/{account_id}"),
            Some(&json!({"name": "  Travel card "}).to_string()),
        )
        .await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    assert_eq!(reply.json["name"], "Travel card");
    assert_eq!(reply.json["transaction_count"], 3);
    assert_eq!(env.get("/accounts").await.json[0]["name"], "Travel card");
    env.finish().await;
}

#[tokio::test]
async fn renaming_needs_a_name_and_an_account() {
    let env = Env::new("ac_rename_bad", vec![two_days()]).await;
    let id = env.upload_one("a.pdf", b"a").await;
    env.confirm_proposed(id, 1).await;
    let account_id = env.get("/accounts").await.json[0]["id"].as_i64().unwrap();

    let empty = env
        .json_call(
            "PATCH",
            &format!("/accounts/{account_id}"),
            Some(r#"{"name":"  "}"#),
        )
        .await;
    let missing = env
        .json_call("PATCH", "/accounts/9999", Some(r#"{"name":"x"}"#))
        .await;

    assert_eq!(
        (empty.status, empty.json["code"].as_str()),
        (400, Some("empty_name"))
    );
    assert_eq!(missing.status, 404);
    env.finish().await;
}
