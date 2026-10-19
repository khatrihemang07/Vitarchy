//! What Finance writes to the log: ids, counts and codes, never a password, a description or
//! an amount.

use crate::common::{
    app::Env,
    results::{card_section, locked, read, unlocked},
    tx,
};
use serde_json::Value;
use vitarchy_core::log_capture::capture_logs;
use vitarchy_finance::reader::testing::FakeReader;

fn lines_with(logs: &str, message: &str) -> Vec<Value> {
    logs.lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|l| l["fields"]["message"] == message)
        .collect()
}

fn card() -> vitarchy_finance::reader::ReaderResult {
    read(vec![card_section(
        "XXXX2003",
        vec![
            tx(2, "SECRETDESCRIPTION", 123_456_789),
            tx(3, "SECRETDESCRIPTION", 987_654_321),
        ],
    )])
}

#[tokio::test]
async fn the_whole_import_flow_logs_counts_but_no_password_description_or_amount() {
    let reader = FakeReader::with_results([locked(), unlocked(card()), card(), card()])
        .writing_decrypted(b"DECRYPTED".to_vec());
    let env = Env::with_reader("log_flow", reader).await;

    let (id, logs) = capture_logs(async {
        env.post(
            "/statement-passwords",
            r#"{"label":"Main","secret":"SECRETPASSWORD42"}"#,
        )
        .await;
        let id = env.upload_one("SECRETFILENAME.pdf", b"ENCRYPTED").await;
        env.post(&format!("/statements/{id}/read-again"), "{}")
            .await;
        env.confirm_proposed(id, 1).await;
        env.post(&format!("/statements/{id}/read-again"), "{}")
            .await;
        env.post(&format!("/statements/{id}/read-again/confirm"), "{}")
            .await;
        env.get("/transactions").await;
        // What the person types in the search is an entered value.
        env.get("/transactions?q=SECRETDESCRIPTION").await;
        env.get("/overview?from=2026-07-01&to=2026-07-31&q=SECRETDESCRIPTION")
            .await;
        id
    })
    .await;

    let confirmed = lines_with(&logs, "statement confirmed");
    assert_eq!(confirmed.len(), 1, "{logs}");
    assert_eq!(confirmed[0]["fields"]["added"], 2);
    assert_eq!(confirmed[0]["fields"]["statement_id"], id);
    assert!(confirmed[0]["fields"]["duration_ms"].is_number());
    assert_eq!(lines_with(&logs, "file read").len(), 1, "{logs}");
    assert_eq!(lines_with(&logs, "read again applied").len(), 1, "{logs}");
    for secret in [
        "SECRETPASSWORD42",
        "SECRETDESCRIPTION",
        "123456789",
        "987654321",
        "SECRETFILENAME",
    ] {
        assert!(!logs.contains(secret), "{secret} leaked: {logs}");
    }
    env.finish().await;
}

#[tokio::test]
async fn a_refused_confirm_logs_its_code_not_what_was_entered() {
    let env = Env::new("log_refused", vec![card()]).await;
    let id = env.upload_one("a.pdf", b"a").await;

    let ((), logs) = capture_logs(async {
        env.post(
            &format!("/statements/{id}/confirm"),
            r#"{"sections":[{"new_account":{"issuer":"SECRETISSUER","kind":"brokerage","name":"SECRETNAME"}}]}"#,
        )
        .await;
    })
    .await;

    let failed = lines_with(&logs, "finance request failed");
    assert_eq!(failed.len(), 1, "{logs}");
    assert_eq!(failed[0]["level"], "WARN");
    assert_eq!(failed[0]["fields"]["code"], "invalid_new_account");
    assert!(
        !logs.contains("SECRETISSUER") && !logs.contains("SECRETNAME"),
        "{logs}"
    );
    env.finish().await;
}
