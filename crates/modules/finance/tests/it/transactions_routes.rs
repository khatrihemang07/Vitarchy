//! The Transactions table's rows.

use crate::common::{
    app::Env,
    results::{app_row, card_section, credit, history, instrument, read, section},
    tx,
};
use vitarchy_finance::reader::contract::AccountKind;

async fn confirmed_env(label: &str) -> Env {
    let mut paid = tx(5, "PAID VIA UPI", 25_000);
    paid.reference = Some("512345678901".to_owned());
    let rows = vec![
        tx(1, "ONE", 100),
        tx(2, "TWO", 200),
        paid,
        credit(9, "NINE", 900),
    ];
    let card = read(vec![card_section("XXXX2003", rows)]);
    let savings = read(vec![section(
        Some("SBI"),
        Some(AccountKind::Savings),
        "XXXX4321",
        vec![tx(3, "SAV", 50)],
    )]);
    let phone = history(vec![instrument(
        None,
        "XXXXXX2003",
        vec![app_row(
            tx(5, "Paid to SOME MERCHANT", 25_000),
            (11, 20),
            Some("P1"),
            Some("512345678901"),
        )],
    )]);
    let env = Env::new(label, vec![card, savings, phone]).await;
    for (name, bytes) in [("a.pdf", &b"a"[..]), ("b.pdf", &b"b"[..])] {
        let id = env.upload_one(name, bytes).await;
        env.confirm_proposed(id, 1).await;
    }
    env
}

fn descriptions(reply: &crate::common::app::Reply) -> Vec<String> {
    reply.json["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["description"].as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
async fn rows_come_newest_first_with_a_total() {
    let env = confirmed_env("tx_all").await;

    let reply = env.get("/transactions").await;

    assert_eq!(reply.json["total"], 5);
    assert_eq!(
        descriptions(&reply),
        ["NINE", "PAID VIA UPI", "SAV", "TWO", "ONE"]
    );
    assert_eq!(reply.json["items"][0]["direction"], "credit");
    assert!(reply.json["items"][0]["time"].is_null());
    assert_eq!(reply.json["items"][0]["no_longer_found"], false);
    env.finish().await;
}

#[tokio::test]
async fn filters_and_paging_narrow_the_rows_and_keep_the_total_honest() {
    let env = confirmed_env("tx_filter").await;
    let accounts = env.get("/accounts").await;
    let card_id = accounts.json[0]["id"].as_i64().unwrap();

    let by_account = env
        .get(&format!("/transactions?account_id={card_id}"))
        .await;
    let by_range = env.get("/transactions?from=2026-07-02&to=2026-07-05").await;
    let paged = env.get("/transactions?limit=2&offset=1").await;

    assert_eq!(by_account.json["total"], 4);
    assert_eq!(descriptions(&by_range), ["PAID VIA UPI", "SAV", "TWO"]);
    assert_eq!(paged.json["total"], 5);
    assert_eq!(descriptions(&paged), ["PAID VIA UPI", "SAV"]);
    env.finish().await;
}

#[tokio::test]
async fn a_payment_apps_row_joins_the_transaction_and_lends_its_time() {
    let env = confirmed_env("tx_app").await;
    let phone = env.upload_one("phone.csv", b"phone").await;
    let confirmed = env.confirm_proposed(phone, 1).await;

    let reply = env.get("/transactions").await;

    assert_eq!(confirmed.status, 200, "{}", confirmed.text);
    assert_eq!(confirmed.json["added"], 0);
    assert_eq!(confirmed.json["already_stored"], 1);
    assert_eq!(reply.json["total"], 5, "the payment is counted once");
    let paid = reply.json["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["description"] == "PAID VIA UPI")
        .unwrap()
        .clone();
    assert_eq!(paid["time"], "11:20:00");
    assert_eq!(paid["printing_count"], 2);
    env.finish().await;
}
