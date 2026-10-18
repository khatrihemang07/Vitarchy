//! `GET /api/time/timeline`: the Timeline analysed for a Period.

mod common;

use common::{
    call, time_db,
    timeline::import_rows,
    timeline_day::{clockify, two_source_day},
};

const DAY: &str = "/api/time/timeline?kind=day&from=2026-10-17&to=2026-10-17&tz=UTC";

#[tokio::test]
async fn a_day_with_both_sources_gives_active_away_unknown_switches_and_focus() {
    let db = time_db("tls_day").await;
    let app = two_source_day(&db).await;

    let reply = call(&app, "GET", DAY, None).await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    let o = &reply.json["overview"];
    assert_eq!(o["active_secs"], 2400);
    assert_eq!(o["away_secs"], 1200);
    assert_eq!(o["unknown_secs"], 300);
    assert_eq!(o["switches"], 1);
    assert_eq!(o["longest_focus_secs"], 1800);
    assert_eq!(o["focus_share"], 0.75);
    assert_eq!(o["first_active"], "2026-10-17T09:00:00Z");
    assert_eq!(reply.json["apps"][0]["app"], "Code");
    assert_eq!(reply.json["hours"][9], 2400);
    db.drop().await;
}

#[tokio::test]
async fn a_day_returns_segments_and_entries_and_a_week_returns_neither() {
    let db = time_db("tls_lists").await;
    let app = two_source_day(&db).await;
    sqlx::query(
        "INSERT INTO time.time_entries (toggl_id, workspace_id, description, tags, start, stop, duration_s, raw)
         VALUES (1, 7, 'Work', '{}', '2026-10-17T09:00Z', '2026-10-17T10:00Z', 3600, '{}')",
    )
    .execute(&db.pool)
    .await
    .unwrap();

    let day = call(&app, "GET", DAY, None).await.json;
    let week = call(
        &app,
        "GET",
        "/api/time/timeline?kind=week&from=2026-10-12&to=2026-10-18&tz=UTC",
        None,
    )
    .await
    .json;

    assert_eq!(day["segments"].as_array().unwrap().len(), 4);
    assert_eq!(day["entries"][0]["description"], "Work");
    assert!(week["segments"].is_null() && week["entries"].is_null());
    assert_eq!(week["days"].as_array().unwrap().len(), 7);
    assert_eq!(week["overview"]["active_secs"], 2400);
    db.drop().await;
}

#[tokio::test]
async fn a_segment_crossing_local_midnight_is_split_between_the_days() {
    let db = time_db("tls_midnight").await;
    let items = [clockify(
        "c1",
        "2026-10-17T23:00:00Z",
        "2026-10-18T01:00:00Z",
        "Code",
        0.0,
    )];
    let (app, ..) = import_rows(&db, &[], &items).await;

    let url = "/api/time/timeline?kind=custom&from=2026-10-17&to=2026-10-18&tz=UTC";
    let reply = call(&app, "GET", url, None).await;

    assert_eq!(reply.json["days"][0]["active_secs"], 3600);
    assert_eq!(reply.json["days"][1]["active_secs"], 3600);
    assert_eq!(reply.json["overview"]["longest_focus_secs"], 7200);
    db.drop().await;
}

#[tokio::test]
async fn the_usual_appears_only_when_there_is_history() {
    let db = time_db("tls_usual").await;
    let items = [
        clockify(
            "c1",
            "2026-10-05T09:00:00Z",
            "2026-10-05T10:00:00Z",
            "Code",
            0.0,
        ),
        clockify(
            "c2",
            "2026-10-06T09:00:00Z",
            "2026-10-06T11:00:00Z",
            "Code",
            0.0,
        ),
        clockify(
            "c3",
            "2026-10-13T09:00:00Z",
            "2026-10-13T10:00:00Z",
            "Code",
            0.0,
        ),
    ];
    let (app, ..) = import_rows(&db, &[], &items).await;

    let week =
        |from: &str, to: &str| format!("/api/time/timeline?kind=week&from={from}&to={to}&tz=UTC");
    let with = call(&app, "GET", &week("2026-10-12", "2026-10-18"), None)
        .await
        .json;
    let without = call(&app, "GET", &week("2026-10-05", "2026-10-11"), None)
        .await
        .json;

    assert_eq!(with["usual"]["days"], 2);
    assert_eq!(with["usual"]["active"]["per_day"]["avg_s"], 5400);
    assert!(without["usual"].is_null());
    db.drop().await;
}

#[tokio::test]
async fn an_empty_period_gives_zeros_and_empty_lists() {
    let db = time_db("tls_empty").await;
    let (app, ..) = import_rows(&db, &[], &[]).await;

    let json = call(
        &app,
        "GET",
        "/api/time/timeline?kind=day&from=2026-01-05&to=2026-01-05&tz=UTC",
        None,
    )
    .await
    .json;

    assert_eq!(json["overview"]["active_secs"], 0);
    assert_eq!(json["overview"]["switches"], 0);
    assert!(json["overview"]["first_active"].is_null());
    assert!(json["usual"].is_null());
    assert_eq!(json["apps"].as_array().unwrap().len(), 0);
    assert_eq!(json["segments"].as_array().unwrap().len(), 0);
    assert_eq!(json["hours"].as_array().unwrap().len(), 24);
    db.drop().await;
}

#[tokio::test]
async fn a_bad_query_is_a_400_naming_the_parameter_but_not_its_value() {
    let db = time_db("tls_bad").await;
    let (app, ..) = import_rows(&db, &[], &[]).await;

    let kind = call(
        &app,
        "GET",
        "/api/time/timeline?kind=bogus&from=2026-10-17&to=2026-10-17&tz=UTC",
        None,
    )
    .await;
    let tz = call(
        &app,
        "GET",
        "/api/time/timeline?kind=day&from=2026-10-17&to=2026-10-17",
        None,
    )
    .await;

    assert_eq!(kind.status, 400);
    assert!(
        kind.text.contains("kind") && !kind.text.contains("bogus"),
        "{}",
        kind.text
    );
    assert_eq!(tz.status, 400);
    assert!(tz.text.contains("tz"), "{}", tz.text);
    db.drop().await;
}

#[tokio::test]
async fn the_usual_counts_every_day_since_the_first_segment_even_before_its_range() {
    let db = time_db("tls_first").await;
    // History starts long before the Usual's range (2026-09-14..10-11), has a gap at the
    // start of that range, and the first Segment inside it is only on 10-05.
    let items = [
        clockify(
            "c0",
            "2026-08-03T09:00:00Z",
            "2026-08-03T10:00:00Z",
            "Code",
            0.0,
        ),
        clockify(
            "c1",
            "2026-10-05T09:00:00Z",
            "2026-10-05T10:00:00Z",
            "Code",
            0.0,
        ),
        clockify(
            "c2",
            "2026-10-13T09:00:00Z",
            "2026-10-13T10:00:00Z",
            "Code",
            0.0,
        ),
    ];
    let (app, ..) = import_rows(&db, &[], &items).await;

    let url = "/api/time/timeline?kind=week&from=2026-10-12&to=2026-10-18&tz=UTC";
    let json = call(&app, "GET", url, None).await.json;

    assert_eq!(json["period"]["usual_available_days"], 28);
    db.drop().await;
}
