//! The Transactions list's filters: Accounts, search, direction and sort.

use crate::common::{
    app::{Env, Reply},
    ledger_world::{account_id, world},
};

fn descriptions(reply: &Reply) -> Vec<String> {
    reply.json["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["description"].as_str().unwrap().to_owned())
        .collect()
}

async fn ask(env: &Env, query: &str) -> Vec<String> {
    let reply = env.get(&format!("/transactions?{query}")).await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    descriptions(&reply)
}

#[tokio::test]
async fn account_id_may_repeat_to_pick_several_accounts() {
    let env = world("txf_accounts").await;
    let cards = format!(
        "account_id={}&account_id={}",
        account_id(&env, "2222").await,
        account_id(&env, "3333").await
    );

    let rows = ask(&env, &cards).await;

    assert_eq!(rows, ["Receipt three", "Receipt two", "Receipt one"]);
    env.finish().await;
}

#[tokio::test]
async fn the_search_ignores_case_and_matches_any_printing() {
    let env = world("txf_search").await;

    assert_eq!(ask(&env, "q=RENT").await, ["Rent payment"]);
    assert_eq!(
        ask(&env, "q=some%20merchant").await,
        ["PAID VIA UPI"],
        "found through the payment app's Printing"
    );
    assert!(ask(&env, "q=nothing%20like%20this").await.is_empty());
    env.finish().await;
}

#[tokio::test]
async fn like_wildcards_in_the_search_match_literally() {
    let env = world("txf_like").await;

    assert_eq!(ask(&env, "q=100%25").await, ["Pays 100% off_set"]);
    assert_eq!(ask(&env, "q=off_set").await, ["Pays 100% off_set"]);
    assert_eq!(
        ask(&env, "q=%25").await.len(),
        1,
        "a lone % is not a wildcard"
    );
    assert!(
        ask(&env, "q=100_%20off").await.is_empty(),
        "_ is not a wildcard"
    );
    env.finish().await;
}

#[tokio::test]
async fn direction_out_keeps_debits_outside_transfers_and_partial_debits() {
    let env = world("txf_out").await;

    let rows = ask(&env, "direction=out").await;

    assert_eq!(
        rows,
        [
            "PAID VIA UPI",
            "Pays twin",
            "Pays 100% off_set",
            "Partial out",
            "Coffee shop",
            "Rent payment"
        ]
    );
    env.finish().await;
}

#[tokio::test]
async fn direction_in_keeps_credits_outside_transfers_and_credits_carrying_an_extra() {
    let env = world("txf_in").await;

    let rows = ask(&env, "direction=in").await;

    assert_eq!(rows, ["Salary July", "Receipt two"]);
    env.finish().await;
}

#[tokio::test]
async fn direction_transfer_keeps_both_sides_of_every_transfer() {
    let env = world("txf_transfer").await;

    let rows = ask(&env, "direction=transfer").await;

    assert_eq!(
        rows,
        [
            "Receipt three",
            "Partial out",
            "Receipt two",
            "Receipt one",
            "Big out"
        ]
    );
    env.finish().await;
}

#[tokio::test]
async fn every_sort_orders_rows_and_ties_break_by_id() {
    let env = world("txf_sort").await;
    let savings = account_id(&env, "1111").await;
    let q = |sort: &str| format!("account_id={savings}&direction=out&sort={sort}");

    assert_eq!(
        ask(&env, &q("date_desc")).await,
        [
            "Pays twin",
            "Pays 100% off_set",
            "Partial out",
            "Coffee shop",
            "Rent payment"
        ]
    );
    assert_eq!(
        ask(&env, &q("date_asc")).await,
        [
            "Rent payment",
            "Coffee shop",
            "Partial out",
            "Pays 100% off_set",
            "Pays twin"
        ]
    );
    // The two debits of 700 tie on amount, so the later id comes first when descending.
    assert_eq!(
        ask(&env, &q("amount_desc")).await,
        [
            "Rent payment",
            "Partial out",
            "Pays twin",
            "Pays 100% off_set",
            "Coffee shop"
        ]
    );
    assert_eq!(
        ask(&env, &q("amount_asc")).await,
        [
            "Coffee shop",
            "Pays 100% off_set",
            "Pays twin",
            "Partial out",
            "Rent payment"
        ]
    );
    env.finish().await;
}

#[tokio::test]
async fn the_total_follows_the_filters() {
    let env = world("txf_total").await;

    let reply = env.get("/transactions?direction=in&limit=1").await;

    assert_eq!(reply.json["total"], 2);
    assert_eq!(reply.json["items"].as_array().unwrap().len(), 1);
    env.finish().await;
}

#[tokio::test]
async fn bad_filters_are_refused() {
    let env = world("txf_bad").await;
    for query in [
        "direction=up",
        "sort=random",
        "account_id=x",
        "from=2026-07-09&to=2026-07-01",
    ] {
        let reply = env.get(&format!("/transactions?{query}")).await;
        assert_eq!(reply.status, 400, "{query}");
        assert_eq!(reply.json["code"], "invalid_query");
    }
    env.finish().await;
}
