//! Which Account a payment app's instrument is: remembered, then matched by issuer and
//! digits across kinds.

use serde_json::json;

use crate::common::{
    app::Env,
    instruments::{account, confirm, file},
};
use vitarchy_finance::reader::contract::AccountKind;

#[tokio::test]
async fn visible_digits_propose_an_account_of_any_kind() {
    let env = Env::new("pa_digits", vec![file(None, "XXXXXX4321", 2)]).await;
    let card = account(
        &env,
        "Bank B",
        AccountKind::CreditCard,
        Some("4123XXXXXXXX4321"),
        "Card",
    )
    .await;

    let id = env.upload_one("phone.csv", b"phone").await;
    let preview = env.get(&format!("/statements/{id}")).await;

    let section = &preview.json["sections"][0];
    assert_eq!(section["matched_account"]["id"], card);
    assert_eq!(section["proposed"]["kind"], json!(null));
    env.finish().await;
}

#[tokio::test]
async fn several_accounts_with_the_same_digits_are_listed_as_candidates() {
    let env = Env::new("pa_candidates", vec![file(None, "XXXXXX4321", 2)]).await;
    account(
        &env,
        "Bank A",
        AccountKind::Savings,
        Some("XXXXXX4321"),
        "One",
    )
    .await;
    account(
        &env,
        "Bank B",
        AccountKind::CreditCard,
        Some("XXXX4321"),
        "Two",
    )
    .await;

    let id = env.upload_one("phone.csv", b"phone").await;
    let preview = env.get(&format!("/statements/{id}")).await;
    let refused = env.confirm_proposed(id, 1).await;

    let section = &preview.json["sections"][0];
    assert!(section["matched_account"].is_null());
    assert_eq!(section["candidate_accounts"].as_array().unwrap().len(), 2);
    assert_eq!(refused.json["code"], "account_ambiguous");
    env.finish().await;
}

#[tokio::test]
async fn a_confirmed_choice_is_remembered_for_the_instrument() {
    let env = Env::new(
        "pa_memory",
        vec![
            file(Some("ICICI Bank"), "ICICI R", 2),
            file(Some("ICICI Bank"), "icici  r", 3),
        ],
    )
    .await;
    account(
        &env,
        "ICICI Bank",
        AccountKind::Savings,
        Some("XXXX6666"),
        "Savings",
    )
    .await;
    let card = account(
        &env,
        "ICICI Bank",
        AccountKind::CreditCard,
        Some("4123XXXXXXXX2003"),
        "Card",
    )
    .await;
    let (_, first) = confirm(&env, "a.csv", json!({"sections": [{"account_id": card}]})).await;
    assert_eq!(first.status, 200, "{}", first.text);

    let second = env.upload(&[("b.csv", b"b")]).await;

    let section = &second.json[0]["sections"][0];
    assert_eq!(section["matched_account"]["id"], card, "{}", second.text);
    assert_eq!(section["candidate_accounts"], json!([]));
    env.finish().await;
}
