//! `GET /transactions/{id}/printings`.

use crate::common::{pairs::confirmed, results::with_balance};

#[tokio::test]
async fn lists_each_statements_copy_in_order_with_its_file_name() {
    let (env, ids) = confirmed(
        "prr_list",
        vec![
            vec![with_balance(5, "UPI/ACME STORE", 500, 9000)],
            vec![with_balance(5, "UPI/ACME STORE/4455 full", 500, 9000)],
        ],
        vec![],
    )
    .await;
    let id = env.get("/transactions").await.json["items"][0]["id"]
        .as_i64()
        .unwrap();

    let reply = env.get(&format!("/transactions/{id}/printings")).await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    let list = reply.json.as_array().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0]["statement_id"], ids[0]);
    assert_eq!(list[0]["file_name"], "file0.pdf");
    assert_eq!(list[0]["description"], "UPI/ACME STORE");
    assert_eq!(list[1]["description"], "UPI/ACME STORE/4455 full");
    assert_eq!(
        (list[1]["balance_paise"].as_i64(), list[1]["gone"].as_bool()),
        (Some(9000), Some(false))
    );
    env.finish().await;
}

#[tokio::test]
async fn an_unknown_transaction_is_a_404() {
    let (env, _) = confirmed("prr_missing", vec![vec![]], vec![]).await;

    let reply = env.get("/transactions/999999/printings").await;

    assert_eq!(
        (reply.status, reply.json["code"].as_str()),
        (404, Some("transaction_not_found"))
    );
    env.finish().await;
}
