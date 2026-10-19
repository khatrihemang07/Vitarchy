//! Transfers across Read again: an unlink is remembered, and rows found later can link.

use crate::common::{
    app::Env,
    results::credit,
    transfers::{import, savings_file, transfer_count, transfer_of},
    tx,
};
use serde_json::json;

fn bank() -> vitarchy_finance::reader::ReaderResult {
    let mut out = tx(8, "NEFT to own account", 4_000_000);
    out.reference = Some("XY12345678".into());
    savings_file("XXXX1111", vec![out])
}

fn other() -> vitarchy_finance::reader::ReaderResult {
    savings_file(
        "XXXX3333",
        vec![credit(8, "NEFT-XY12345678-SOMEONE", 4_000_000)],
    )
}

#[tokio::test]
async fn an_unlinked_transfer_stays_unlinked_after_the_statement_is_read_again() {
    let env = Env::new("tra_one", vec![bank(), other(), other(), other()]).await;
    import(&env, "bank").await;
    let other_id = import(&env, "other").await;
    let id = transfer_of(&env, "NEFT to own account").await["id"]
        .as_i64()
        .unwrap();
    assert_eq!(
        env.json_call("DELETE", &format!("/transfers/{id}"), None)
            .await
            .status,
        204
    );

    let again = env
        .post(&format!("/statements/{other_id}/read-again/confirm"), "{}")
        .await;

    assert_eq!(again.status, 200, "{}", again.text);
    assert_eq!(transfer_count(&env).await, 0);
    assert!(transfer_of(&env, "NEFT to own account").await.is_null());
    env.finish().await;
}

#[tokio::test]
async fn read_all_again_links_a_transfer_that_a_newly_found_row_makes_possible() {
    let results = vec![
        savings_file("XXXX1111", vec![tx(8, "pay out", 4_000_000)]),
        savings_file("XXXX3333", vec![credit(2, "something else", 99)]),
        savings_file("XXXX1111", vec![tx(8, "pay out", 4_000_000)]),
        savings_file(
            "XXXX3333",
            vec![
                credit(2, "something else", 99),
                credit(8, "arrived", 4_000_000),
            ],
        ),
    ];
    let env = Env::new("tra_all", results).await;
    import(&env, "one").await;
    import(&env, "two").await;
    assert_eq!(transfer_count(&env).await, 0);

    let reply = env
        .post("/statements/read-again/confirm", &json!({}).to_string())
        .await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    assert_eq!(transfer_of(&env, "pay out").await["rung"], "unique_amount");
    env.finish().await;
}
