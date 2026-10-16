//! What Time writes to the log: counts and codes, never the token or entry text.

mod common;

use common::{
    FakeToggl, TOKEN, call, connect,
    snapshot::{Fixture, Row},
    stopped_entry, time_app, time_app_with_snapshot, time_db,
};
use serde_json::Value;
use vitarchy_core::log_capture::capture_logs;
use vitarchy_time::toggl::error::TogglError;

fn lines_with(logs: &str, message: &str) -> Vec<Value> {
    logs.lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|l| l["fields"]["message"] == message)
        .collect()
}

#[tokio::test]
async fn a_sync_logs_counts_and_duration_but_not_the_token_or_descriptions() {
    let db = time_db("logsync").await;
    let fake = FakeToggl::new();
    fake.with(|w| w.entries = vec![stopped_entry(1, "SECRETDESCRIPTION")]);
    let app = time_app(&db, &fake);

    let (_, logs) = capture_logs(async {
        connect(&app).await;
        call(&app, "POST", "/api/time/sync", None).await
    })
    .await;

    let finished = lines_with(&logs, "sync finished");
    assert_eq!(finished.len(), 1, "{logs}");
    assert_eq!(finished[0]["fields"]["inserted"], 1);
    assert!(finished[0]["fields"]["duration_ms"].is_number());
    assert!(
        !logs.contains(TOKEN) && !logs.contains("SECRETDESCRIPTION"),
        "{logs}"
    );
    db.drop().await;
}

#[tokio::test]
async fn a_failed_sync_logs_its_error_code_as_a_warning() {
    let db = time_db("logfail").await;
    let fake = FakeToggl::new();
    let app = time_app(&db, &fake);
    connect(&app).await;
    fake.with(|w| {
        w.fail_with = Some(TogglError::RateLimited {
            resets_in_secs: Some(5),
        });
    });

    let (_, logs) = capture_logs(call(&app, "POST", "/api/time/sync", None)).await;

    let failed = lines_with(&logs, "sync failed");
    assert_eq!(failed[0]["level"], "WARN", "{logs}");
    assert_eq!(failed[0]["fields"]["error_code"], "rate_limited");
    assert!(!logs.contains(TOKEN), "{logs}");
    db.drop().await;
}

#[tokio::test]
async fn a_rejected_token_is_not_logged() {
    let db = time_db("logbadtoken").await;
    let app = time_app(&db, &FakeToggl::new());

    let (_, logs) = capture_logs(call(
        &app,
        "PUT",
        "/api/time/connection",
        Some(r#"{"api_token":"WRONGTOKEN999"}"#),
    ))
    .await;

    assert!(!logs.contains("WRONGTOKEN999"), "{logs}");
    db.drop().await;
}

#[tokio::test]
async fn an_import_logs_counts_and_duration_but_not_titles_or_app_names() {
    let db = time_db("logimport").await;
    let snap = Fixture::new();
    let mut row = Row::new(1, 0.0, 60.0);
    row.app = "SECRETAPPNAME";
    row.title = "SECRETWINDOWTITLE";
    snap.write(&[row]);
    let app = time_app_with_snapshot(&db, &FakeToggl::new(), snap.file());

    let (_, logs) = capture_logs(call(&app, "POST", "/api/time/activity/import", None)).await;

    let finished = lines_with(&logs, "activity import finished");
    assert_eq!(finished.len(), 1, "{logs}");
    assert_eq!(finished[0]["fields"]["inserted"], 1);
    assert!(finished[0]["fields"]["duration_ms"].is_number());
    assert!(
        !logs.contains("SECRETAPPNAME") && !logs.contains("SECRETWINDOWTITLE"),
        "{logs}"
    );
    db.drop().await;
}
