//! Linking Transfers by hand, unlinking them, and the candidates the dialog offers.

use serde_json::{Value, json};

use crate::common::{
    app::Env,
    results::credit,
    transfers::{card_file, import, items, savings_file, transfer_count, transfer_of},
    tx,
};

/// Three Accounts: a savings Account paying 10,000 on day 5 with no markers or references,
/// a card that received 4,000 and another that received 6,000, and an unrelated debit.
async fn world(label: &str) -> Env {
    let env = Env::new(
        label,
        vec![
            savings_file(
                "XXXX1111",
                vec![
                    tx(5, "payment out", 1_000_000),
                    tx(20, "elsewhere", 321_000),
                ],
            ),
            card_file("XXXX2222", vec![credit(6, "credit one", 400_000)]),
            card_file("XXXX4444", vec![credit(9, "credit two", 600_000)]),
        ],
    )
    .await;
    for name in ["bank", "card one", "card two"] {
        import(&env, name).await;
    }
    env
}

async fn id_of(env: &Env, description: &str) -> i64 {
    items(env)
        .await
        .into_iter()
        .find(|item| item["description"] == description)
        .map(|item| item["id"].as_i64().unwrap())
        .unwrap()
}

async fn link(env: &Env, debit: &str, credits: &[&str]) -> crate::common::app::Reply {
    let mut ids = Vec::new();
    for credit in credits {
        ids.push(id_of(env, credit).await);
    }
    let body = json!({"debit_id": id_of(env, debit).await, "credit_ids": ids});
    env.post("/transfers", &body.to_string()).await
}

#[tokio::test]
async fn several_credits_can_be_linked_by_hand_to_one_debit() {
    let env = world("trr_link").await;
    assert_eq!(transfer_count(&env).await, 0);

    let reply = link(&env, "payment out", &["credit one", "credit two"]).await;

    assert_eq!(reply.status, 201, "{}", reply.text);
    assert_eq!(reply.json["transfer"]["rung"], "manual");
    assert_eq!(reply.json["transfer"]["partial"], false);
    let row = transfer_of(&env, "credit two").await;
    assert_eq!(row["id"], reply.json["transfer"]["id"]);
    env.finish().await;
}

#[tokio::test]
async fn a_partial_link_reports_what_is_still_to_arrive() {
    let env = world("trr_partial").await;

    let reply = link(&env, "payment out", &["credit one"]).await;

    assert_eq!(reply.status, 201, "{}", reply.text);
    assert_eq!(reply.json["transfer"]["partial"], true);
    assert_eq!(reply.json["transfer"]["remaining_paise"], 600_000);
    env.finish().await;
}

#[tokio::test]
async fn credits_adding_up_to_much_more_than_the_debit_are_refused() {
    let env = world("trr_much").await;
    let reply = link(&env, "elsewhere", &["credit one"]).await;
    assert_eq!(reply.status, 400, "{}", reply.text);
    assert_eq!(reply.json["code"], "transfer_too_much");
    assert_eq!(transfer_count(&env).await, 0);
    env.finish().await;
}

#[tokio::test]
async fn up_to_fifty_rupees_over_the_debit_is_allowed_and_counts_as_extra() {
    let env = Env::new(
        "trr_extra",
        vec![
            savings_file(
                "XXXX1111",
                vec![tx(5, "small out", 395_000), tx(5, "bigger out", 394_999)],
            ),
            card_file("XXXX2222", vec![credit(6, "slightly more in", 400_000)]),
        ],
    )
    .await;
    import(&env, "bank").await;
    import(&env, "card").await;

    let ok = link(&env, "small out", &["slightly more in"]).await;
    assert_eq!(ok.status, 201, "{}", ok.text);
    assert_eq!(ok.json["transfer"]["extra_paise"], 5_000);
    let transfer = ok.json["transfer"]["id"].as_i64().unwrap();
    env.json_call("DELETE", &format!("/transfers/{transfer}"), None)
        .await;

    let over = link(&env, "bigger out", &["slightly more in"]).await;
    assert_eq!(
        (over.status, over.json["code"].as_str()),
        (400, Some("transfer_too_much"))
    );
    env.finish().await;
}

#[tokio::test]
async fn refusals_name_their_code() {
    let env = world("trr_refuse").await;
    let debit = id_of(&env, "payment out").await;
    let one = id_of(&env, "credit one").await;

    let cases: [(Value, u16, &str); 5] = [
        (
            json!({"debit_id": debit, "credit_ids": []}),
            400,
            "transfer_no_credits",
        ),
        (
            json!({"debit_id": one, "credit_ids": [one]}),
            400,
            "transfer_wrong_direction",
        ),
        (
            json!({"debit_id": debit, "credit_ids": [debit]}),
            400,
            "transfer_wrong_direction",
        ),
        (
            json!({"debit_id": 999_999, "credit_ids": [one]}),
            404,
            "transaction_not_found",
        ),
        (
            json!({"debit_id": debit, "credit_ids": [one, one]}),
            400,
            "transfer_duplicate_credit",
        ),
    ];
    for (body, status, code) in cases {
        let reply = env.post("/transfers", &body.to_string()).await;
        assert_eq!(
            (reply.status, reply.json["code"].as_str()),
            (status, Some(code)),
            "{}",
            reply.text
        );
    }
    env.finish().await;
}

#[tokio::test]
async fn a_credit_on_the_debits_own_account_is_refused() {
    let env = Env::new(
        "trr_same",
        vec![savings_file(
            "XXXX1111",
            vec![tx(5, "out", 100), credit(5, "in", 100)],
        )],
    )
    .await;
    import(&env, "bank").await;
    let reply = link(&env, "out", &["in"]).await;
    assert_eq!(reply.json["code"], "transfer_same_account");
    env.finish().await;
}

#[tokio::test]
async fn a_transaction_already_in_a_transfer_cannot_be_linked_again() {
    let env = world("trr_twice").await;
    assert_eq!(link(&env, "payment out", &["credit one"]).await.status, 201);

    let again = link(&env, "payment out", &["credit two"]).await;

    assert_eq!(again.status, 409, "{}", again.text);
    assert_eq!(again.json["code"], "transfer_already_linked");
    env.finish().await;
}

#[tokio::test]
async fn unlinking_removes_the_transfer_from_both_rows() {
    let env = world("trr_unlink").await;
    let made = link(&env, "payment out", &["credit one", "credit two"]).await;
    let id = made.json["transfer"]["id"].as_i64().unwrap();

    let gone = env
        .json_call("DELETE", &format!("/transfers/{id}"), None)
        .await;

    assert_eq!(gone.status, 204);
    assert!(transfer_of(&env, "payment out").await.is_null());
    assert!(transfer_of(&env, "credit one").await.is_null());
    let again = env
        .json_call("DELETE", &format!("/transfers/{id}"), None)
        .await;
    assert_eq!(
        (again.status, again.json["code"].as_str()),
        (404, Some("transfer_not_found"))
    );
    env.finish().await;
}

#[tokio::test]
async fn an_unlinked_pair_can_still_be_linked_by_hand() {
    let env = world("trr_relink").await;
    let id = link(&env, "payment out", &["credit one"]).await.json["transfer"]["id"]
        .as_i64()
        .unwrap();
    env.json_call("DELETE", &format!("/transfers/{id}"), None)
        .await;

    let reply = link(&env, "payment out", &["credit one"]).await;

    assert_eq!(reply.status, 201, "{}", reply.text);
    env.finish().await;
}

#[tokio::test]
async fn candidates_for_a_debit_are_the_credits_that_fit_within_a_week() {
    let env = world("trr_cands").await;
    let debit = id_of(&env, "payment out").await;

    let reply = env
        .get(&format!("/transactions/{debit}/transfer-candidates"))
        .await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    let found: Vec<&str> = reply
        .json
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["description"].as_str().unwrap())
        .collect();
    assert_eq!(found, vec!["credit one", "credit two"], "nearest first");
    env.finish().await;
}

#[tokio::test]
async fn candidates_for_a_credit_are_debits_that_could_have_paid_it() {
    let env = world("trr_cands2").await;
    let credit = id_of(&env, "credit one").await;

    let reply = env
        .get(&format!("/transactions/{credit}/transfer-candidates"))
        .await;

    let found: Vec<&str> = reply
        .json
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["description"].as_str().unwrap())
        .collect();
    assert_eq!(
        found,
        vec!["payment out"],
        "the debit 15 days away is out of reach"
    );
    env.finish().await;
}

#[tokio::test]
async fn linked_transactions_are_not_offered_and_an_unknown_one_is_a_404() {
    let env = world("trr_cands3").await;
    link(&env, "payment out", &["credit one"]).await;
    let debit = id_of(&env, "payment out").await;
    let two = id_of(&env, "credit two").await;

    assert!(
        env.get(&format!("/transactions/{debit}/transfer-candidates"))
            .await
            .json
            .as_array()
            .unwrap()
            .is_empty()
    );
    let for_two = env
        .get(&format!("/transactions/{two}/transfer-candidates"))
        .await;
    assert!(
        for_two.json.as_array().unwrap().is_empty(),
        "the only debit is taken"
    );
    let missing = env.get("/transactions/999999/transfer-candidates").await;
    assert_eq!(
        (missing.status, missing.json["code"].as_str()),
        (404, Some("transaction_not_found"))
    );
    env.finish().await;
}

#[tokio::test]
async fn credits_beyond_the_debit_are_allowed_only_on_credit_cards() {
    let env = Env::new(
        "trr_cards_only",
        vec![
            savings_file("XXXX1111", vec![tx(5, "sent out", 395_000)]),
            savings_file("XXXX3333", vec![credit(6, "arrived savings", 400_000)]),
            card_file("XXXX2222", vec![credit(6, "arrived card", 400_000)]),
        ],
    )
    .await;
    for name in ["bank", "other bank", "card"] {
        import(&env, name).await;
    }
    let debit = id_of(&env, "sent out").await;

    let listed = env
        .get(&format!("/transactions/{debit}/transfer-candidates"))
        .await;
    let found: Vec<&str> = listed
        .json
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["description"].as_str().unwrap())
        .collect();
    assert_eq!(
        found,
        vec!["arrived card"],
        "the savings credit would exceed the debit"
    );

    let refused = link(&env, "sent out", &["arrived savings"]).await;
    assert_eq!(
        (refused.status, refused.json["code"].as_str()),
        (400, Some("transfer_too_much"))
    );
    let mixed = link(&env, "sent out", &["arrived savings", "arrived card"]).await;
    assert_eq!(mixed.json["code"], "transfer_too_much");
    let ok = link(&env, "sent out", &["arrived card"]).await;
    assert_eq!(ok.status, 201, "{}", ok.text);
    assert_eq!(ok.json["transfer"]["extra_paise"], 5_000);
    env.finish().await;
}

#[tokio::test]
async fn a_transaction_a_read_again_no_longer_finds_cannot_be_linked() {
    let env = world("trr_gone").await;
    sqlx::query("UPDATE finance.transactions SET no_longer_found_at = now() WHERE description = 'credit one'")
        .execute(&env.db.pool)
        .await
        .unwrap();

    let reply = link(&env, "payment out", &["credit one"]).await;

    assert_eq!(
        (reply.status, reply.json["code"].as_str()),
        (409, Some("transfer_no_longer_found"))
    );
    assert_eq!(transfer_count(&env).await, 0);
    env.finish().await;
}
