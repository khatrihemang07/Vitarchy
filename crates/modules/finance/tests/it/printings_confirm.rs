//! Confirm keeps a Printing from every Statement and shows the fullest (ADR 0015).

use crate::common::{
    pairs::{confirmed, uploaded},
    results::with_balance,
    tx,
};

const CUT: &str = "UPI/ACME STORE/4455/payment for the order";
const FULL: &str = "UPI/ACME STORE/4455/payment for the order of nine items";

async fn items(env: &crate::common::app::Env) -> Vec<serde_json::Value> {
    env.get("/transactions").await.json["items"]
        .as_array()
        .unwrap()
        .clone()
}

#[tokio::test]
async fn pdf_then_xlsx_shows_the_full_description_with_two_printings_and_the_same_amount() {
    let (env, _) = confirmed(
        "pr_pdf_xlsx",
        vec![
            vec![with_balance(5, CUT, 500, 9000)],
            vec![with_balance(5, FULL, 500, 9000)],
        ],
        vec![],
    )
    .await;

    let rows = items(&env).await;

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["description"], FULL);
    assert_eq!(rows[0]["printing_count"], 2);
    assert_eq!(rows[0]["amount_paise"], 500);
    assert_eq!(rows[0]["balance_paise"], 9000);
    env.finish().await;
}

#[tokio::test]
async fn xlsx_then_pdf_keeps_the_full_text() {
    let (env, _) = confirmed(
        "pr_xlsx_pdf",
        vec![
            vec![with_balance(5, FULL, 500, 9000)],
            vec![with_balance(5, CUT, 500, 9000)],
        ],
        vec![],
    )
    .await;

    let rows = items(&env).await;

    assert_eq!(
        (
            rows[0]["description"].as_str(),
            rows[0]["printing_count"].as_i64()
        ),
        (Some(FULL), Some(2))
    );
    env.finish().await;
}

#[tokio::test]
async fn a_description_that_does_not_extend_the_first_keeps_the_first() {
    let (env, _) = confirmed(
        "pr_other_text",
        vec![
            vec![with_balance(5, "UPI/ACME STORE/4455", 500, 9000)],
            vec![with_balance(5, "ACME STORE CARD PURCHASE", 500, 9000)],
        ],
        vec![],
    )
    .await;

    let rows = items(&env).await;

    assert_eq!(rows[0]["description"], "UPI/ACME STORE/4455");
    assert_eq!(rows[0]["printing_count"], 2);
    env.finish().await;
}

#[tokio::test]
async fn gaps_are_filled_from_a_later_printing() {
    let (env, _) = confirmed(
        "pr_gaps",
        vec![
            vec![tx(5, CUT, 500)],
            vec![with_balance(5, FULL, 500, 9000)],
        ],
        vec![],
    )
    .await;

    let rows = items(&env).await;

    assert_eq!(rows[0]["balance_paise"], 9000);
    assert_eq!(rows[0]["description"], FULL);
    env.finish().await;
}

#[tokio::test]
async fn repeated_identical_rows_stay_separate_and_keep_one_printing_each() {
    let twice = vec![
        tx(5, "coffee shop visit", 100),
        tx(5, "coffee shop visit", 100),
    ];
    let (env, _) = confirmed("pr_twins", vec![twice.clone(), twice], vec![]).await;

    let rows = items(&env).await;

    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|r| r["printing_count"] == 1));
    env.finish().await;
}

#[tokio::test]
async fn confirm_counts_rows_that_gained_detail() {
    let (env, ids) = uploaded(
        "pr_counts",
        vec![
            vec![with_balance(5, CUT, 500, 9000), tx(6, "fresh row", 70)],
            vec![with_balance(5, FULL, 500, 9000)],
        ],
        vec![],
    )
    .await;
    env.confirm_proposed(ids[0], 1).await;

    let second = env.confirm_proposed(ids[1], 1).await;

    assert_eq!(second.json["added"], 0);
    assert_eq!(second.json["already_stored"], 1);
    assert_eq!(second.json["enriched"], 1);
    env.finish().await;
}

#[tokio::test]
async fn the_preview_counts_and_tags_rows_that_would_gain_detail() {
    let (env, ids) = uploaded(
        "pr_preview",
        vec![
            vec![
                with_balance(5, CUT, 500, 9000),
                with_balance(6, "same text", 70, 8930),
            ],
            vec![
                with_balance(5, FULL, 500, 9000),
                with_balance(6, "same text", 70, 8930),
            ],
        ],
        vec![],
    )
    .await;
    env.confirm_proposed(ids[0], 1).await;

    let reply = env.get(&format!("/statements/{}", ids[1])).await;

    let section = &reply.json["sections"][0];
    assert_eq!(
        (
            section["new_count"].as_i64(),
            section["already_stored_count"].as_i64(),
            section["enriched_count"].as_i64()
        ),
        (Some(0), Some(2), Some(1))
    );
    assert_eq!(section["transactions"][0]["gains_detail"], true);
    assert_eq!(section["transactions"][1]["gains_detail"], false);
    env.finish().await;
}

#[tokio::test]
async fn a_confirmed_statement_previews_nothing_to_gain() {
    let (env, ids) = confirmed(
        "pr_preview_done",
        vec![
            vec![with_balance(5, CUT, 500, 9000)],
            vec![with_balance(5, FULL, 500, 9000)],
        ],
        vec![],
    )
    .await;

    let reply = env.get(&format!("/statements/{}", ids[1])).await;

    assert_eq!(reply.json["sections"][0]["enriched_count"], 0);
    env.finish().await;
}
