//! What saving settings writes to the log: scopes and key names, never values.

mod common;

use common::{call, settings_app, settings_db};
use serde_json::Value;
use vitarchy_core::log_capture::capture_logs;

/// Parsed JSON log lines whose `message` equals `message`.
fn lines_with(logs: &str, message: &str) -> Vec<Value> {
    logs.lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|l| l["fields"]["message"] == message)
        .collect()
}

#[tokio::test]
async fn a_save_logs_its_scope_and_changed_keys_but_not_the_values() {
    let db = settings_db("logsave").await;
    let app = settings_app(&db);
    call(
        &app,
        "PUT",
        "/api/settings/core",
        Some(r#"{"display_name":"OLDSECRET"}"#),
    )
    .await;

    let (_, logs) = capture_logs(call(
        &app,
        "PUT",
        "/api/settings/core",
        Some(r#"{"display_name":"OLDSECRET","theme":"dark"}"#),
    ))
    .await;

    let saved = lines_with(&logs, "settings saved");
    assert_eq!(saved.len(), 1, "{logs}");
    assert_eq!(saved[0]["fields"]["scope"], "core");
    assert_eq!(saved[0]["fields"]["changed"], r#"["theme"]"#);
    assert!(!logs.contains("OLDSECRET"), "{logs}");
    db.drop().await;
}

#[tokio::test]
async fn a_save_costs_one_statement_so_there_is_no_separate_read_before_the_write() {
    let db = settings_db("onestmt").await;
    let app = settings_app(&db);

    let (_, logs) = capture_logs(call(
        &app,
        "PUT",
        "/api/settings/core",
        Some(r#"{"theme":"dark"}"#),
    ))
    .await;

    let statements = logs
        .lines()
        .filter(|l| l.contains("\"sqlx::query\""))
        .count();
    assert_eq!(statements, 1, "{logs}");
    db.drop().await;
}

#[tokio::test]
async fn a_rejected_enum_value_is_logged_as_a_warning_without_the_value() {
    let db = settings_db("logreject").await;
    let app = settings_app(&db);

    let (reply, logs) = capture_logs(call(
        &app,
        "PUT",
        "/api/settings/core",
        Some(r#"{"theme":"purple"}"#),
    ))
    .await;

    assert_eq!(reply.status, 400);
    let rejected = lines_with(&logs, "request rejected");
    assert_eq!(rejected.len(), 1, "{logs}");
    assert_eq!(rejected[0]["level"], "WARN");
    assert_eq!(rejected[0]["fields"]["status"], 400);
    assert!(!logs.contains("purple"), "{logs}");
    db.drop().await;
}

#[tokio::test]
async fn an_invalid_module_id_never_reaches_the_log() {
    let db = settings_db("logbadid").await;
    let app = settings_app(&db);

    let (reply, logs) = capture_logs(call(
        &app,
        "PUT",
        "/api/settings/modules/Evil%0AInjected",
        Some(r#"{"a":1}"#),
    ))
    .await;

    assert_eq!(reply.status, 400);
    assert!(
        !logs.contains("Evil") && !logs.contains("Injected"),
        "{logs}"
    );
    db.drop().await;
}

#[tokio::test]
async fn a_valid_module_id_is_logged_as_the_scope() {
    let db = settings_db("logmodid").await;
    let app = settings_app(&db);

    let (_, logs) = capture_logs(call(
        &app,
        "PUT",
        "/api/settings/modules/time-tracker",
        Some(r#"{"goal":8}"#),
    ))
    .await;

    let saved = lines_with(&logs, "settings saved");
    assert_eq!(saved[0]["fields"]["scope"], "time-tracker", "{logs}");
    db.drop().await;
}

#[tokio::test]
async fn a_failed_save_is_a_500_that_logs_no_success_and_no_values() {
    let db = settings_db("logfail").await;
    let app = settings_app(&db);
    db.pool.close().await;

    let (reply, logs) = capture_logs(call(
        &app,
        "PUT",
        "/api/settings/core",
        Some(r#"{"display_name":"SECRETNAME"}"#),
    ))
    .await;

    assert_eq!(reply.status, 500);
    assert!(lines_with(&logs, "settings saved").is_empty(), "{logs}");
    assert_eq!(lines_with(&logs, "request failed")[0]["level"], "ERROR");
    assert!(!logs.contains("SECRETNAME"), "{logs}");
    db.drop().await;
}
