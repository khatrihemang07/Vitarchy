//! The Transactions table's rows.

use crate::common::{
    app::Env,
    payment,
    results::{card_section, credit, history, read, section},
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
    let mut record = payment(Some("P1"), Some("512345678901"), 25_000);
    record.instrument_digits = Some("2003".to_owned());
    let env = Env::new(label, vec![card, savings, history(vec![record])]).await;
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
    assert!(reply.json["items"][0]["payment_record"].is_null());
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
async fn a_linked_payment_record_comes_inline() {
    let env = confirmed_env("tx_linked").await;
    let phone = env.upload_one("phone.csv", b"phone").await;
    let confirmed = env.confirm_proposed(phone, 0).await;

    let reply = env.get("/transactions").await;

    assert_eq!(confirmed.json["linked"], 1);
    let linked = reply.json["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| !r["payment_record"].is_null())
        .unwrap()
        .clone();
    assert_eq!(linked["description"], "PAID VIA UPI");
    assert_eq!(linked["payment_record"]["utr"], "512345678901");
    assert_eq!(
        linked["payment_record"]["description"],
        "Paid to SOME MERCHANT"
    );
    assert_eq!(
        linked["payment_record"]["occurred_at"],
        "2026-07-05T11:20:00"
    );
    env.finish().await;
}

#[tokio::test]
async fn invalid_filters_are_refused_with_a_code() {
    let env = confirmed_env("tx_invalid").await;

    for query in [
        "limit=0",
        "limit=501",
        "offset=-1",
        "from=2026-07-09&to=2026-07-01",
        "from=yesterday",
        "account_id=abc",
    ] {
        let reply = env.get(&format!("/transactions?{query}")).await;
        assert_eq!(
            (reply.status, reply.json["code"].as_str()),
            (400, Some("invalid_query")),
            "{query}"
        );
    }
    env.finish().await;
}
