//! Listing and reading Statements, and discarding an unconfirmed one.

use crate::common::{
    app::Env,
    results::{card_section, read},
    tx,
};
use vitarchy_finance::{
    files::sha256_hex,
    repo::statements::{self, NewStatement},
};

fn card() -> vitarchy_finance::reader::ReaderResult {
    read(vec![card_section(
        "XXXX2003",
        vec![tx(2, "SHOP", 100), tx(3, "CAFE", 200)],
    )])
}

#[tokio::test]
async fn the_list_filters_by_status_and_summarises_each_file() {
    let env = Env::new("st_list", vec![card()]).await;
    let first = env.upload_one("a.pdf", b"one").await;
    let second = env.upload_one("b.pdf", b"two").await;
    env.confirm_proposed(first, 1).await;

    let all = env.get("/statements").await;
    let waiting = env.get("/statements?status=unconfirmed").await;
    let done = env.get("/statements?status=confirmed").await;

    assert_eq!(all.json.as_array().unwrap().len(), 2);
    assert_eq!(all.json[0]["id"], second, "newest first");
    assert_eq!(waiting.json.as_array().unwrap().len(), 1);
    assert_eq!(waiting.json[0]["id"], second);
    assert_eq!(done.json[0]["id"], first);
    assert_eq!(done.json[0]["status"], "confirmed");
    assert_eq!(
        (
            done.json[0]["account_count"].as_i64(),
            done.json[0]["row_count"].as_i64()
        ),
        (Some(1), Some(2))
    );
    assert!(done.json[0]["confirmed_at"].is_string());
    env.finish().await;
}

#[tokio::test]
async fn an_unknown_status_is_refused_with_a_code() {
    let env = Env::new("st_badstatus", vec![card()]).await;

    let reply = env.get("/statements?status=bogus").await;

    assert_eq!(
        (reply.status, reply.json["code"].as_str()),
        (400, Some("invalid_query"))
    );
    env.finish().await;
}

#[tokio::test]
async fn one_statement_can_be_read_back_as_a_preview() {
    let env = Env::new("st_get", vec![card()]).await;
    let id = env.upload_one("a.pdf", b"one").await;

    let reply = env.get(&format!("/statements/{id}")).await;

    assert_eq!(reply.status, 200);
    assert_eq!(reply.json["id"], id);
    assert_eq!(reply.json["duplicate_file"], false);
    assert_eq!(reply.json["sections"][0]["transaction_count"], 2);
    env.finish().await;
}

#[tokio::test]
async fn a_missing_statement_is_a_404_with_a_code() {
    let env = Env::new("st_404", vec![card()]).await;

    for (method, uri) in [
        ("GET", "/statements/999"),
        ("DELETE", "/statements/999"),
        ("POST", "/statements/999/read-again"),
        ("POST", "/statements/999/read-again/confirm"),
    ] {
        let reply = env.json_call(method, uri, None).await;
        assert_eq!(
            (reply.status, reply.json["code"].as_str()),
            (404, Some("statement_not_found")),
            "{method} {uri}"
        );
    }
    env.finish().await;
}

#[tokio::test]
async fn discarding_an_unconfirmed_statement_removes_its_row_and_file() {
    let env = Env::new("st_discard", vec![card()]).await;
    let id = env.upload_one("a.pdf", b"discard me").await;
    let file = env.stored_file(&sha256_hex(b"discard me"));
    assert!(file.exists());

    let reply = env
        .json_call("DELETE", &format!("/statements/{id}"), None)
        .await;

    assert_eq!(reply.status, 204);
    assert!(!file.exists());
    assert_eq!(env.get(&format!("/statements/{id}")).await.status, 404);
    env.finish().await;
}

#[tokio::test]
async fn an_unconfirmed_statement_that_already_printed_transactions_cannot_be_discarded() {
    let env = Env::new("st_discard_printed", vec![card()]).await;
    let id = env.upload_one("a.pdf", b"reopened").await;
    env.confirm_proposed(id, 1).await;
    // What the payment-app migration leaves for a history it reopened: unconfirmed, yet still printing.
    sqlx::query("UPDATE finance.statements SET status = 'unconfirmed', confirmed_at = NULL")
        .execute(&env.db.pool)
        .await
        .unwrap();

    let reply = env
        .json_call("DELETE", &format!("/statements/{id}"), None)
        .await;

    assert_eq!(
        (reply.status, reply.json["code"].as_str()),
        (409, Some("statement_has_rows"))
    );
    env.finish().await;
}

#[tokio::test]
async fn discarding_keeps_a_file_another_statement_still_uses() {
    let env = Env::new("st_discard_shared", vec![card()]).await;
    let id = env.upload_one("a.pdf", b"shared").await;
    let sha = sha256_hex(b"shared");
    statements::insert(
        &env.db.pool,
        &NewStatement {
            file_sha256: &sha,
            file_name: "twin.pdf",
            format: Some("pdf"),
            kind: None,
            outcome: "locked",
            result: None,
            reader_version: None,
        },
    )
    .await
    .unwrap();

    let reply = env
        .json_call("DELETE", &format!("/statements/{id}"), None)
        .await;

    assert_eq!(reply.status, 204);
    assert!(env.stored_file(&sha).exists());
    env.finish().await;
}

#[tokio::test]
async fn a_confirmed_statement_cannot_be_discarded() {
    let env = Env::new("st_discard_confirmed", vec![card()]).await;
    let id = env.upload_one("a.pdf", b"keep me").await;
    env.confirm_proposed(id, 1).await;

    let reply = env
        .json_call("DELETE", &format!("/statements/{id}"), None)
        .await;

    assert_eq!(
        (reply.status, reply.json["code"].as_str()),
        (409, Some("statement_confirmed"))
    );
    assert!(env.stored_file(&sha256_hex(b"keep me")).exists());
    assert_eq!(env.get(&format!("/statements/{id}")).await.status, 200);
    env.finish().await;
}
