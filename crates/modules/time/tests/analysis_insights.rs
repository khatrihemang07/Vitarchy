//! `GET /api/time/analysis/insights`: plain statistics over stored entries, Activity and Routines.

mod common;

use axum::Router;
use chrono::{Datelike, NaiveDate, Weekday};
use common::{FakeToggl, call, time_app, time_db};
use serde_json::{Value, json};
use vitarchy_core::test_db::TestDb;

const MAY: &str = "from=2026-05-01&to=2026-05-31&tz=UTC";

async fn entry(db: &TestDb, description: &str, start: &str, stop: &str) {
    sqlx::query(
        "INSERT INTO time.time_entries (toggl_id, workspace_id, project_id, description, start, stop, duration_s, raw)
         VALUES ((random() * 1e12)::bigint, 7, NULL, $1, $2::timestamptz, $3::timestamptz,
                 extract(epoch FROM $3::timestamptz - $2::timestamptz)::bigint, '{}')",
    )
    .bind(description)
    .bind(start)
    .bind(stop)
    .execute(&db.pool)
    .await
    .unwrap();
}

async fn activity(db: &TestDb, app: &str, start: &str, end: &str) {
    sqlx::query(
        "INSERT INTO time.activities (toggl_id, app, title, start_at, end_at, idle, raw)
         VALUES (gen_random_uuid(), $1, '', $2::timestamptz, $3::timestamptz, false, '{}')",
    )
    .bind(app)
    .bind(start)
    .bind(end)
    .execute(&db.pool)
    .await
    .unwrap();
}

fn weekdays(from: &str, to: &str) -> Vec<NaiveDate> {
    let (from, to): (NaiveDate, NaiveDate) = (from.parse().unwrap(), to.parse().unwrap());
    from.iter_days()
        .take_while(|d| *d <= to)
        .filter(|d| !matches!(d.weekday(), Weekday::Sat | Weekday::Sun))
        .collect()
}

async fn get(app: &Router, uri: &str) -> (u16, Value) {
    let reply = call(app, "GET", uri, None).await;
    (reply.status, reply.json)
}

/// Four weekday weeks of office life in May 2026 and, before them, a shorter commute and less
/// screen time; one very long Tuesday, shorter Fridays and a workout habit.
async fn seeded(label: &str) -> (TestDb, Router) {
    let db = time_db(label).await;
    let app = time_app(&db, &FakeToggl::new());
    let steps = json!([
        {"label": "To office", "kind": "description", "value": "travel to_office"},
        {"label": "At office", "kind": "description", "value": "inside_office"},
    ]);
    let body = json!({"name": "Office day", "steps": steps}).to_string();
    assert_eq!(
        call(&app, "POST", "/api/time/routines", Some(&body))
            .await
            .status,
        201
    );
    for day in weekdays("2026-04-20", "2026-05-29") {
        let at = |t: &str| format!("{day}T{t}:00Z");
        let early = day < "2026-05-04".parse().unwrap();
        entry(
            &db,
            "travel to_office",
            &at("08:00"),
            &at(if early { "08:20" } else { "08:30" }),
        )
        .await;
        let leave = match (day.weekday(), day.to_string().as_str()) {
            (_, "2026-05-12") => "19:12",
            (Weekday::Fri, _) => "16:10",
            _ => "17:00",
        };
        entry(&db, "inside_office", &at("09:00"), &at(leave)).await;
        activity(
            &db,
            "Brave",
            &at("10:00"),
            &at(if early { "11:00" } else { "12:00" }),
        )
        .await;
        activity(&db, "Slack", &at("13:00"), &at("13:10")).await;
        if day.day() % 2 == 0 && !early {
            entry(&db, "workout", &at("18:30"), &at("19:10")).await;
        }
    }
    (db, app)
}

fn find<'a>(body: &'a Value, kind: &str, name: &str, measure: &str) -> &'a Value {
    body["insights"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["kind"] == kind && i["subject"]["name"] == name && i["measure"] == measure)
        .unwrap_or_else(|| panic!("no {kind} insight on {name} {measure}: {body}"))
}

#[tokio::test]
async fn every_kind_is_found_in_a_realistic_month_and_ranked_by_score() {
    let (db, app) = seeded("insights_all").await;
    let (status, body) = get(&app, &format!("/api/time/analysis/insights?{MAY}")).await;
    assert_eq!(status, 200, "{body}");

    let steady = find(&body, "typical_time", "To office", "start");
    assert_eq!(steady["numbers"]["value"], 480);
    assert_eq!(steady["subject"]["kind"], "routine_step");
    assert_eq!(steady["subject"]["routine"]["name"], "Office day");
    assert!(steady["subject"]["routine"]["id"].is_i64());
    assert_eq!(body["change_window_days"], 28);

    assert_eq!(
        find(&body, "typical_duration", "To office", "length")["numbers"]["value"],
        30
    );

    let friday = find(&body, "weekday_difference", "At office", "end");
    assert_eq!(friday["numbers"]["weekday"], 4);
    assert_eq!(friday["numbers"]["delta"], -50);

    assert_eq!(
        find(&body, "change", "To office", "length")["numbers"]["delta"],
        10
    );
    let brave = find(&body, "change", "Brave", "length");
    assert_eq!(brave["subject"]["kind"], "app");
    assert_eq!(brave["numbers"]["delta"], 60);

    let long = find(&body, "outlier", "At office", "length");
    assert_eq!(long["numbers"]["value"], 612);
    assert_eq!(long["days"], json!(["2026-05-12"]));

    assert_eq!(
        find(&body, "typical_time", "workout", "start")["subject"]["kind"],
        "description"
    );

    let scores: Vec<f64> = body["insights"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["score"].as_f64().unwrap())
        .collect();
    assert!(scores.windows(2).all(|w| w[0] >= w[1]), "{scores:?}");
    assert!(scores.len() <= 20);
    db.drop().await;
}

#[tokio::test]
async fn a_short_period_still_compares_the_last_four_weeks_with_the_four_before() {
    let (db, app) = seeded("insights_short").await;
    let (status, body) = get(
        &app,
        "/api/time/analysis/insights?from=2026-05-29&to=2026-05-29&tz=UTC",
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        find(&body, "change", "Brave", "length")["numbers"]["delta"],
        60
    );
    db.drop().await;
}

#[tokio::test]
async fn a_short_period_still_gets_changes_for_a_habit_of_the_whole_reach() {
    let db = time_db("insights_short_habit").await;
    let app = time_app(&db, &FakeToggl::new());
    // A stretch every day: 20 minutes until May 3, 40 minutes after.
    for day in weekdays("2026-04-06", "2026-05-29") {
        let long = day >= "2026-05-04".parse().unwrap();
        let stop = if long { "07:40" } else { "07:20" };
        entry(
            &db,
            "stretch",
            &format!("{day}T07:00:00Z"),
            &format!("{day}T{stop}:00Z"),
        )
        .await;
    }
    let (status, body) = get(
        &app,
        "/api/time/analysis/insights?from=2026-05-29&to=2026-05-29&tz=UTC",
    )
    .await;
    assert_eq!(status, 200, "{body}");
    let change = find(&body, "change", "stretch", "length");
    assert_eq!(change["numbers"]["delta"], 20);
    assert_eq!(change["subject"]["routine"], Value::Null);
    db.drop().await;
}

#[tokio::test]
async fn an_empty_period_has_no_insights() {
    let (db, app) = seeded("insights_empty").await;
    let (status, body) = get(
        &app,
        "/api/time/analysis/insights?from=2020-01-01&to=2020-01-31&tz=UTC",
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["insights"], json!([]));
    db.drop().await;
}

#[tokio::test]
async fn a_bad_zone_is_a_coded_bad_request() {
    let (db, app) = seeded("insights_bad_zone").await;
    let (status, body) = get(
        &app,
        "/api/time/analysis/insights?from=2026-05-01&to=2026-05-31&tz=Nowhere/Land",
    )
    .await;
    assert_eq!(status, 400);
    assert_eq!(body["code"], "unknown_time_zone");
    db.drop().await;
}
