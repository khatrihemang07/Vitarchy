//! Confirming chooses, creates or remembers the Account each section belongs to.

use crate::common::{
    account,
    app::Env,
    cards::{MASK, card},
    results::{read, section},
    tx,
};
use serde_json::json;
use vitarchy_finance::reader::contract::AccountKind;

#[tokio::test]
async fn an_unrecognised_account_needs_a_new_account_from_the_person() {
    let unknown = section(None, None, "XXXX7777", vec![tx(2, "A", 100)]);
    let env = Env::new("cf_new_account", vec![read(vec![unknown])]).await;
    let id = env.upload_one("a.pdf", b"a").await;

    let refused = env.confirm_proposed(id, 1).await;
    let body = json!({"sections": [{"new_account": {"issuer": "Example Bank", "kind": "savings", "name": " Salary "}}]});
    let created = env
        .post(&format!("/statements/{id}/confirm"), &body.to_string())
        .await;

    assert_eq!(
        (refused.status, refused.json["code"].as_str()),
        (400, Some("new_account_required"))
    );
    assert_eq!(created.status, 200, "{}", created.text);
    let account = &env.get("/accounts").await.json[0];
    assert_eq!(
        (
            account["name"].as_str(),
            account["issuer"].as_str(),
            account["kind"].as_str()
        ),
        (Some("Salary"), Some("Example Bank"), Some("savings"))
    );
    assert_eq!(account["mask"], "XXXX7777");
    env.finish().await;
}

#[tokio::test]
async fn assigning_to_another_account_is_remembered_for_next_time() {
    let env = Env::new(
        "cf_reassign",
        vec![card(vec![tx(2, "A", 100)]), card(vec![tx(3, "B", 200)])],
    )
    .await;
    let other = account(&env.db, "XXXX5555").await;
    let first = env.upload_one("a.pdf", b"a").await;
    let body = json!({"sections": [{"account_id": other.id}]});

    let reply = env
        .post(&format!("/statements/{first}/confirm"), &body.to_string())
        .await;
    let second = env.upload(&[("b.pdf", b"b")]).await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    assert_eq!(
        second.json[0]["sections"][0]["matched_account"]["id"],
        other.id
    );
    assert_eq!(env.get("/accounts").await.json.as_array().unwrap().len(), 1);
    let remembered: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM finance.account_identifiers WHERE account_id = $1 AND digits = '2003'",
    )
    .bind(other.id)
    .fetch_one(&env.db.pool)
    .await
    .unwrap();
    assert_eq!(remembered, 1);
    env.finish().await;
}

#[tokio::test]
async fn several_matching_accounts_need_a_choice() {
    let env = Env::new("cf_ambiguous", vec![card(vec![tx(2, "A", 100)])]).await;
    for name in ["One", "Two"] {
        vitarchy_finance::repo::accounts::insert(
            &env.db.pool,
            "ICICI Bank",
            AccountKind::CreditCard,
            Some(MASK),
            name,
        )
        .await
        .unwrap();
    }
    let id = env.upload_one("a.pdf", b"a").await;

    let preview = env.get(&format!("/statements/{id}")).await;
    let reply = env.confirm_proposed(id, 1).await;

    assert_eq!(
        preview.json["sections"][0]["candidate_accounts"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(preview.json["sections"][0]["matched_account"].is_null());
    assert_eq!(
        (reply.status, reply.json["code"].as_str()),
        (400, Some("account_ambiguous"))
    );
    env.finish().await;
}

#[tokio::test]
async fn a_new_account_that_is_already_known_reuses_it() {
    let env = Env::new(
        "cf_new_known",
        vec![card(vec![tx(2, "A", 100)]), card(vec![tx(3, "B", 200)])],
    )
    .await;
    let uploaded = env.upload(&[("a.pdf", b"a"), ("b.pdf", b"b")]).await;
    let new = json!({"sections": [{"new_account": {"issuer": "ICICI Bank", "kind": "credit_card", "name": "Card"}}]});

    for upload in uploaded.json.as_array().unwrap() {
        let id = upload["id"].as_i64().unwrap();
        let reply = env
            .post(&format!("/statements/{id}/confirm"), &new.to_string())
            .await;
        assert_eq!(reply.status, 200, "{}", reply.text);
    }

    assert_eq!(env.get("/accounts").await.json.as_array().unwrap().len(), 1);
    env.finish().await;
}

#[tokio::test]
async fn a_new_account_without_a_number_is_created_even_when_one_is_known() {
    let env = Env::new(
        "cf_new_unnumbered",
        vec![read(vec![section(
            Some("ICICI Bank"),
            Some(AccountKind::CreditCard),
            "",
            vec![tx(2, "A", 100)],
        )])],
    )
    .await;
    account(&env.db, "XXXX2003").await;
    let id = env.upload_one("a.pdf", b"a").await;
    let new = json!({"sections": [{"new_account": {"issuer": "ICICI Bank", "kind": "credit_card", "name": "Other"}}]});

    let reply = env
        .post(&format!("/statements/{id}/confirm"), &new.to_string())
        .await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    assert_eq!(env.get("/accounts").await.json.as_array().unwrap().len(), 2);
    env.finish().await;
}
