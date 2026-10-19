//! Hidden Accounts: their Transactions leave the page and they take no part in Transfers.

use crate::common::{
    app::Env,
    results::credit,
    transfers::{import, items, savings_file, transfer_count, transfer_of},
    tx,
};
use serde_json::json;

async fn two_linked(label: &str) -> (Env, i64, i64) {
    let env = Env::new(
        label,
        vec![
            savings_file(
                "XXXX1111",
                vec![tx(6, "money out", 777_700), tx(9, "kept row", 100)],
            ),
            savings_file("XXXX3333", vec![credit(6, "money in", 777_700)]),
        ],
    )
    .await;
    import(&env, "one").await;
    import(&env, "two").await;
    let accounts = env.get("/accounts").await.json;
    let id = |mask: &str| {
        accounts
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["mask"] == mask)
            .unwrap()["id"]
            .as_i64()
            .unwrap()
    };
    let (first, second) = (id("XXXX1111"), id("XXXX3333"));
    (env, first, second)
}

async fn hide(env: &Env, id: i64, hidden: bool) -> crate::common::app::Reply {
    env.json_call(
        "PATCH",
        &format!("/accounts/{id}"),
        Some(&json!({"hidden": hidden}).to_string()),
    )
    .await
}

#[tokio::test]
async fn accounts_are_visible_until_hidden_and_the_list_says_which() {
    let (env, first, second) = two_linked("trh_flag").await;
    assert!(
        env.get("/accounts")
            .await
            .json
            .as_array()
            .unwrap()
            .iter()
            .all(|a| a["hidden"] == false)
    );

    let reply = hide(&env, second, true).await;

    assert_eq!(
        (reply.status, reply.json["hidden"].clone()),
        (200, true.into()),
        "{}",
        reply.text
    );
    let accounts = env.get("/accounts").await.json;
    let hidden: Vec<i64> = accounts
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| a["hidden"] == true)
        .map(|a| a["id"].as_i64().unwrap())
        .collect();
    assert_eq!(hidden, vec![second], "a hidden Account is still listed");
    assert_ne!(first, second);
    env.finish().await;
}

#[tokio::test]
async fn a_hidden_accounts_transactions_leave_the_transactions_page() {
    let (env, first, _) = two_linked("trh_page").await;
    assert_eq!(items(&env).await.len(), 3);

    hide(&env, first, true).await;

    let page = env.get("/transactions").await.json;
    assert_eq!(page["total"], 1);
    assert_eq!(page["items"][0]["description"], "money in");
    let only_hidden = env
        .get(&format!("/transactions?account_id={first}"))
        .await
        .json;
    assert_eq!(only_hidden["total"], 0);
    env.finish().await;
}

#[tokio::test]
async fn hiding_an_account_dissolves_its_transfers_and_showing_it_links_them_again() {
    let (env, first, _) = two_linked("trh_dissolve").await;
    assert_eq!(transfer_count(&env).await, 1);

    hide(&env, first, true).await;
    assert_eq!(transfer_count(&env).await, 0);

    hide(&env, first, false).await;
    assert_eq!(transfer_count(&env).await, 1);
    assert_eq!(
        transfer_of(&env, "money out").await["rung"],
        "unique_amount"
    );
    env.finish().await;
}

#[tokio::test]
async fn a_hidden_account_is_never_linked_by_a_later_statement() {
    let env = Env::new(
        "trh_later",
        vec![
            savings_file("XXXX3333", vec![credit(6, "money in", 777_700)]),
            savings_file("XXXX1111", vec![tx(6, "money out", 777_700)]),
        ],
    )
    .await;
    import(&env, "hidden one").await;
    let account = env.get("/accounts").await.json[0]["id"].as_i64().unwrap();
    hide(&env, account, true).await;

    import(&env, "other").await;

    assert_eq!(transfer_count(&env).await, 0);
    env.finish().await;
}

#[tokio::test]
async fn a_transaction_of_a_hidden_account_cannot_be_linked_by_hand() {
    let (env, _, second) = two_linked("trh_manual").await;
    hide(&env, second, true).await;
    let id_of = |description: &'static str| {
        let pool = env.db.pool.clone();
        async move {
            sqlx::query_scalar::<_, i64>(
                "SELECT id FROM finance.transactions WHERE description = $1",
            )
            .bind(description)
            .fetch_one(&pool)
            .await
            .unwrap()
        }
    };
    let body =
        json!({"debit_id": id_of("money out").await, "credit_ids": [id_of("money in").await]});

    let reply = env.post("/transfers", &body.to_string()).await;

    assert_eq!(
        (reply.status, reply.json["code"].as_str()),
        (409, Some("transfer_hidden_account"))
    );
    env.finish().await;
}

#[tokio::test]
async fn patching_nothing_or_a_missing_account_is_refused() {
    let (env, first, _) = two_linked("trh_refuse").await;
    let nothing = env
        .json_call("PATCH", &format!("/accounts/{first}"), Some("{}"))
        .await;
    assert_eq!(
        (nothing.status, nothing.json["code"].as_str()),
        (400, Some("nothing_to_change"))
    );
    let missing = hide(&env, 999_999, true).await;
    assert_eq!(missing.status, 404);
    env.finish().await;
}
