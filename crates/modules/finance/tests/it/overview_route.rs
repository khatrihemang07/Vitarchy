//! `GET /overview`: Money in, Money out, Net and Transfers, with the previous period.

use serde_json::{Value, json};

use crate::common::{
    app::Env,
    ledger_world::{account_id, world},
};

async fn overview(env: &Env, query: &str) -> Value {
    let reply = env.get(&format!("/overview?{query}")).await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    reply.json
}

fn figures(o: &Value, name: &str) -> (i64, i64) {
    (
        o[name]["paise"].as_i64().unwrap(),
        o[name]["previous_paise"].as_i64().unwrap(),
    )
}

const JULY: &str = "from=2026-07-01&to=2026-07-31";

#[tokio::test]
async fn transfers_are_left_out_but_an_extra_and_a_remainder_count() {
    let env = world("ov_all").await;

    let o = overview(&env, JULY).await;

    // In: the Salary and the 40 Extra. Out: the plain debits and the 1,500 still to come.
    assert_eq!(figures(&o, "money_in"), (5_004_000, 0));
    assert_eq!(figures(&o, "money_out"), (1_285_000, 0));
    assert_eq!(figures(&o, "net"), (3_719_000, 0));
    assert_eq!(figures(&o, "transfers_total"), (1_050_000, 0));
    env.finish().await;
}

#[tokio::test]
async fn daily_lists_every_day_and_puts_the_extra_and_remainder_on_their_dates() {
    let env = world("ov_daily").await;

    let o = overview(&env, JULY).await;
    let daily = o["daily"].as_array().unwrap();
    let day = |n: usize| {
        (
            daily[n - 1]["money_in_paise"].as_i64().unwrap(),
            daily[n - 1]["money_out_paise"].as_i64().unwrap(),
        )
    };

    assert_eq!(daily.len(), 31);
    assert_eq!(daily[0]["date"], "2026-07-01");
    assert_eq!(day(1), (0, 0));
    assert_eq!(day(8), (4_000, 0), "the Extra, on the credit's date");
    assert_eq!(day(15), (0, 150_000), "the remainder, on the debit's date");
    assert_eq!(day(12), (5_000_000, 0));
    env.finish().await;
}

#[tokio::test]
async fn the_previous_period_is_the_range_of_equal_length_before() {
    let env = world("ov_previous").await;

    let o = overview(&env, "from=2026-07-11&to=2026-07-20").await;

    assert_eq!(figures(&o, "money_in"), (5_000_000, 4_000));
    assert_eq!(figures(&o, "money_out"), (220_000, 925_000));
    assert_eq!(figures(&o, "transfers_total"), (50_000, 1_000_000));
    assert_eq!(figures(&o, "net"), (4_780_000, 4_000 - 925_000));
    env.finish().await;
}

#[tokio::test]
async fn one_side_of_a_transfer_still_counts_as_a_transfer() {
    let env = world("ov_side").await;
    let cards = account_id(&env, "2222").await;
    let savings = account_id(&env, "1111").await;

    let card_side = overview(&env, &format!("{JULY}&account_id={cards}")).await;
    let savings_side = overview(&env, &format!("{JULY}&account_id={savings}")).await;

    assert_eq!(
        figures(&card_side, "money_in"),
        (4_000, 0),
        "only the Extra"
    );
    assert_eq!(figures(&card_side, "money_out"), (0, 0));
    assert_eq!(figures(&card_side, "transfers_total"), (1_000_000, 0));
    assert_eq!(figures(&savings_side, "money_in"), (5_000_000, 0));
    assert_eq!(figures(&savings_side, "transfers_total"), (1_050_000, 0));
    env.finish().await;
}

#[tokio::test]
async fn the_search_narrows_the_figures() {
    let env = world("ov_search").await;

    let o = overview(&env, &format!("{JULY}&q=RENT")).await;

    assert_eq!(figures(&o, "money_out"), (900_000, 0));
    assert_eq!(figures(&o, "money_in"), (0, 0));
    env.finish().await;
}

#[tokio::test]
async fn hidden_accounts_are_left_out() {
    let env = world("ov_hidden").await;
    let savings = account_id(&env, "1111").await;
    env.json_call(
        "PATCH",
        &format!("/accounts/{savings}"),
        Some(&json!({"hidden": true}).to_string()),
    )
    .await;

    let o = overview(&env, JULY).await;

    assert_eq!(
        figures(&o, "money_out"),
        (70_000, 0),
        "only the card's UPI payment"
    );
    env.finish().await;
}

#[tokio::test]
async fn a_missing_backwards_or_too_long_range_is_refused() {
    let env = world("ov_bad").await;
    for query in [
        "",
        "from=2026-07-01",
        "from=2026-07-09&to=2026-07-01",
        "from=2025-01-01&to=2026-07-01",
    ] {
        let reply = env.get(&format!("/overview?{query}")).await;
        assert_eq!(reply.status, 400, "{query}");
        assert_eq!(reply.json["code"], "invalid_query");
    }
    env.finish().await;
}
