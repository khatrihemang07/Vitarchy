//! A payment app's Instrument the person has to place: a new Account is created once, and
//! each Instrument of a file is confirmed on its own Account.

use serde_json::json;

use crate::common::{
    app::Env,
    instruments::{account, confirm, file},
    results::{history, instrument},
    tx,
};
use vitarchy_finance::reader::contract::AccountKind;

#[tokio::test]
async fn an_unrecognised_instrument_needs_a_kind_from_the_person_and_is_then_remembered() {
    let env = Env::new(
        "pa_new",
        vec![file(None, "BOB R", 2), file(None, "BOB R", 3)],
    )
    .await;
    let id = env.upload_one("a.csv", b"a").await;

    let refused = env.confirm_proposed(id, 1).await;
    let body = json!({"sections": [{"new_account": {"issuer": "Example Bank", "kind": "credit_card", "name": "BOB card"}}]});
    let created = env
        .post(&format!("/statements/{id}/confirm"), &body.to_string())
        .await;
    let next = env.upload(&[("b.csv", b"b")]).await;

    assert_eq!(refused.json["code"], "new_account_required");
    assert_eq!(created.status, 200, "{}", created.text);
    let made = &env.get("/accounts").await.json[0];
    assert_eq!(
        (made["kind"].as_str(), made["name"].as_str()),
        (Some("credit_card"), Some("BOB card"))
    );
    assert!(
        made["mask"].is_null(),
        "a label such as BOB R is not a number"
    );
    assert_eq!(
        next.json[0]["sections"][0]["matched_account"]["id"],
        made["id"]
    );
    env.finish().await;
}

#[tokio::test]
async fn a_new_account_for_digits_that_match_one_account_is_that_account() {
    let env = Env::new("pa_no_duplicate", vec![file(None, "XXXXXX4321", 2)]).await;
    let existing = account(
        &env,
        "Example Bank",
        AccountKind::Savings,
        Some("XXXXXX4321"),
        "Salary",
    )
    .await;
    let body = json!({"sections": [{"new_account": {"issuer": "Example Bank", "kind": "savings", "name": "Again"}}]});

    let (_, reply) = confirm(&env, "a.csv", body).await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    let listed = env.get("/accounts").await;
    assert_eq!(listed.json.as_array().unwrap().len(), 1);
    assert_eq!(listed.json[0]["id"], existing);
    env.finish().await;
}

#[tokio::test]
async fn each_instrument_of_a_file_is_confirmed_on_its_own_account() {
    let two = history(vec![
        instrument(None, "XXXXXX1111", vec![tx(2, "Paid to ONE", 100)]),
        instrument(None, "XXXXXX2222", vec![tx(3, "Paid to TWO", 200)]),
    ]);
    let env = Env::new("pa_two_sections", vec![two]).await;
    let one = account(
        &env,
        "Bank A",
        AccountKind::Savings,
        Some("XXXXXX1111"),
        "One",
    )
    .await;
    let two = account(
        &env,
        "Bank B",
        AccountKind::Savings,
        Some("XXXXXX2222"),
        "Two",
    )
    .await;

    let id = env.upload_one("phone.csv", b"phone").await;
    let reply = env.confirm_proposed(id, 2).await;

    assert_eq!(reply.json["added"], 2, "{}", reply.text);
    let by_account = |account: i64| format!("/transactions?account_id={account}");
    assert_eq!(env.get(&by_account(one)).await.json["total"], 1);
    assert_eq!(env.get(&by_account(two)).await.json["total"], 1);
    env.finish().await;
}
