//! `/api/time/analysis/activity/...`: the Activity overview and timeline over stored rows.

mod common;

use axum::Router;
use common::{FakeToggl, call, time_app, time_db};
use serde_json::Value;
use vitarchy_core::test_db::TestDb;

/// Stores one synthetic row (times are RFC 3339 text).
async fn store(db: &TestDb, app: &str, title: &str, start: &str, end: &str) {
    sqlx::query(
        "INSERT INTO time.activities (toggl_id, app, title, start_at, end_at, idle, raw)
         VALUES (gen_random_uuid(), $1, $2, $3::timestamptz, $4::timestamptz, false, '{}')",
    )
    .bind(app)
    .bind(title)
    .bind(start)
    .bind(end)
    .execute(&db.pool)
    .await
    .unwrap();
}

async fn get(app: &Router, uri: &str) -> (u16, Value) {
    let reply = call(app, "GET", uri, None).await;
    (reply.status, reply.json)
}

/// One part (`totals`, `heatmap`, `bounds` or `trend`) of the overview for a query string.
async fn part(app: &Router, query: &str, name: &str) -> Value {
    let (status, body) = get(
        app,
        &format!("/api/time/analysis/activity/overview?{query}"),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    body[name].clone()
}

const UTC_WEEK: &str = "from=2026-05-04&to=2026-05-10&tz=UTC";

/// A small realistic week: editor work, a browser, a midnight-crossing row and a 111 h row.
async fn seeded(label: &str) -> (TestDb, Router) {
    let db = time_db(label).await;
    store(
        &db,
        "Code",
        "main.rs — vitarchy",
        "2026-05-04T09:00:00Z",
        "2026-05-04T10:00:00Z",
    )
    .await;
    store(
        &db,
        "Code",
        "lib.rs — vitarchy",
        "2026-05-04T10:00:30Z",
        "2026-05-04T10:30:00Z",
    )
    .await;
    store(
        &db,
        "Brave Browser",
        "Lofi - YouTube",
        "2026-05-04T12:00:00Z",
        "2026-05-04T12:20:00Z",
    )
    .await;
    store(
        &db,
        "Brave Browser",
        "",
        "2026-05-04T12:20:00Z",
        "2026-05-04T12:25:00Z",
    )
    .await;
    // Crosses UTC midnight: 2 h on the 5th, 1 h on the 6th.
    store(
        &db,
        "\u{200E}WhatsApp",
        "",
        "2026-05-05T22:00:00Z",
        "2026-05-06T01:00:00Z",
    )
    .await;
    // The 111 h Spotify row: starts the 7th 20:00Z, ends the 12th 11:00Z.
    store(
        &db,
        "Spotify",
        "",
        "2026-05-07T20:00:00Z",
        "2026-05-12T11:00:00Z",
    )
    .await;
    let app = time_app(&db, &FakeToggl::new());
    (db, app)
}

fn seconds_of(item: &Value) -> i64 {
    item["seconds"].as_i64().unwrap()
}

fn find<'a>(apps: &'a [Value], name: &str) -> &'a Value {
    apps.iter().find(|a| a["app"] == name).unwrap()
}

#[tokio::test]
async fn totals_rank_apps_contexts_and_titles_and_split_at_midnight() {
    let (db, app) = seeded("act_totals").await;
    let body = part(&app, UTC_WEEK, "totals").await;
    let apps = body["apps"].as_array().unwrap();
    // Spotify: 4 h on the 7th, then the 8th, 9th and 10th in full (the Period ends the 10th).
    assert_eq!(apps[0]["app"], "Spotify");
    assert_eq!(seconds_of(&apps[0]), 76 * 3600);
    let code = find(apps, "Code");
    assert_eq!(seconds_of(code), 3600 + 1770);
    assert_eq!(code["contexts"][0]["name"], "vitarchy");
    assert_eq!(
        code["contexts"][0]["titles"][0]["title"],
        "main.rs — vitarchy"
    );
    // The invisible mark is trimmed and the row is split across two Days: 3 h in all.
    assert_eq!(seconds_of(find(apps, "WhatsApp")), 3 * 3600);
    let brave = find(apps, "Brave Browser");
    assert_eq!(brave["contexts"][0]["name"], "YouTube");
    assert_eq!(brave["contexts"][1]["name"], "Brave Browser");
    let sum: i64 = apps.iter().map(seconds_of).sum();
    assert_eq!(body["total_seconds"].as_i64().unwrap(), sum);
    db.drop().await;
}

#[tokio::test]
async fn the_overview_ranks_contexts_and_lists_each_finished_day_with_activity() {
    let (db, app) = seeded("act_ranked").await;
    let overview = part(&app, UTC_WEEK, "top_contexts").await;
    let names: Vec<&str> = overview
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert_eq!(names.first(), Some(&"vitarchy"));
    // An app that is its own Context is already in the apps list.
    assert!(!names.contains(&"Spotify"));
    let days = part(&app, UTC_WEEK, "day_totals").await;
    let days = days.as_array().unwrap();
    assert!(!days.is_empty() && days.len() <= 7);
    let total: i64 = days.iter().map(seconds_of).sum();
    assert_eq!(
        total,
        part(&app, UTC_WEEK, "totals").await["total_seconds"]
            .as_i64()
            .unwrap()
    );
    db.drop().await;
}

#[tokio::test]
async fn totals_follow_the_requested_time_zone() {
    let (db, app) = seeded("act_tz").await;
    // 22:00Z on the 5th is 03:30 on the 6th in Kolkata, so the whole row is on the 6th there.
    let body = part(
        &app,
        "from=2026-05-06&to=2026-05-06&tz=Asia/Kolkata",
        "totals",
    )
    .await;
    let apps = body["apps"].as_array().unwrap();
    assert_eq!(seconds_of(find(apps, "WhatsApp")), 3 * 3600);
    let body = part(&app, "from=2026-05-06&to=2026-05-06&tz=UTC", "totals").await;
    assert_eq!(
        seconds_of(find(body["apps"].as_array().unwrap(), "WhatsApp")),
        3600
    );
    db.drop().await;
}

#[tokio::test]
async fn an_empty_period_is_empty_not_an_error() {
    let (db, app) = seeded("act_empty").await;
    let q = "from=2020-01-01&to=2020-01-07&tz=UTC";
    let (status, overview) = get(&app, &format!("/api/time/analysis/activity/overview?{q}")).await;
    let totals = &overview["totals"];
    assert_eq!((status, totals["total_seconds"].as_i64()), (200, Some(0)));
    assert!(totals["apps"].as_array().unwrap().is_empty());
    assert_eq!(overview["heatmap"]["seconds"].as_array().unwrap().len(), 7);
    assert!(overview["bounds"].as_array().unwrap().is_empty());
    let (_, lane) = get(
        &app,
        "/api/time/analysis/activity/timeline?day=2020-01-01&tz=UTC",
    )
    .await;
    assert!(lane["spans"].as_array().unwrap().is_empty());
    assert!(overview["trend"]["series"].as_array().unwrap().is_empty());
    db.drop().await;
}

#[tokio::test]
async fn the_heatmap_puts_seconds_in_weekday_and_hour_cells() {
    let (db, app) = seeded("act_heat").await;
    let body = part(&app, UTC_WEEK, "heatmap").await;
    let grid = &body["seconds"];
    // Monday 4 May: Code 09:00-10:30 (30 s gap between rows is not counted).
    assert_eq!(grid[0][9], 3600);
    assert_eq!(grid[0][10], 1770);
    // Tuesday 22:00-24:00 and Wednesday 00:00-01:00 hold the midnight-crossing row.
    assert_eq!(
        (
            grid[1][22].as_i64(),
            grid[1][23].as_i64(),
            grid[2][0].as_i64()
        ),
        (Some(3600), Some(3600), Some(3600))
    );
    db.drop().await;
}

#[tokio::test]
async fn bounds_give_first_and_last_local_minutes_of_each_active_day() {
    let (db, app) = seeded("act_bounds").await;
    let body = part(&app, UTC_WEEK, "bounds").await;
    let days: Vec<(String, i64, i64)> = body
        .as_array()
        .unwrap()
        .iter()
        .map(|d| {
            (
                d["date"].as_str().unwrap().to_owned(),
                d["first_minute"].as_i64().unwrap(),
                d["last_minute"].as_i64().unwrap(),
            )
        })
        .collect();
    assert_eq!(days[0], ("2026-05-04".to_owned(), 9 * 60, 12 * 60 + 25));
    assert_eq!(days[1], ("2026-05-05".to_owned(), 22 * 60, 24 * 60));
    assert_eq!(days[2], ("2026-05-06".to_owned(), 0, 60));
    assert_eq!(days.len(), 7);
    db.drop().await;
}

#[tokio::test]
async fn the_timeline_merges_close_rows_and_is_clipped_to_the_day() {
    let (db, app) = seeded("act_lane").await;
    let (_, lane) = get(
        &app,
        "/api/time/analysis/activity/timeline?day=2026-05-04&tz=UTC",
    )
    .await;
    let spans = lane["spans"].as_array().unwrap();
    assert_eq!(spans.len(), 3);
    assert_eq!(
        (
            spans[0]["context"].as_str(),
            spans[0]["start"].as_str(),
            spans[0]["end"].as_str()
        ),
        (
            Some("vitarchy"),
            Some("2026-05-04T09:00:00Z"),
            Some("2026-05-04T10:30:00Z")
        )
    );
    assert_eq!(lane["day_start"], "2026-05-04T00:00:00Z");
    let (_, next) = get(
        &app,
        "/api/time/analysis/activity/timeline?day=2026-05-06&tz=UTC",
    )
    .await;
    assert_eq!(next["spans"][0]["end"], "2026-05-06T01:00:00Z");
    assert_eq!(next["spans"][0]["start"], "2026-05-06T00:00:00Z");
    db.drop().await;
}

#[tokio::test]
async fn the_trend_buckets_by_the_requested_week_start() {
    let (db, app) = seeded("act_trend").await;
    let monday = part(&app, &format!("{UTC_WEEK}&week_start=monday"), "trend").await;
    assert_eq!(monday["weeks"].as_array().unwrap().len(), 1);
    assert_eq!(monday["series"][0]["app"], "Spotify");
    assert_eq!(monday["series"][0]["seconds"][0], 76 * 3600);
    let sunday = part(&app, &format!("{UTC_WEEK}&week_start=sunday"), "trend").await;
    assert_eq!(sunday["weeks"][0]["start"], "2026-05-03");
    assert_eq!(sunday["weeks"][0]["days"], 6);
    assert_eq!(sunday["weeks"][1]["days"], 1);
    let (status, _) = get(
        &app,
        &format!("/api/time/analysis/activity/overview?{UTC_WEEK}&week_start=friday"),
    )
    .await;
    assert_eq!(status, 400);
    db.drop().await;
}

#[tokio::test]
async fn bad_parameters_are_coded_400s_on_every_route() {
    let (db, app) = seeded("act_bad").await;
    let uri = "/api/time/analysis/activity/overview?from=2026-05-04&to=2026-05-10&tz=Nowhere/Land";
    let reply = call(&app, "GET", uri, None).await;
    assert_eq!(
        (reply.status, reply.json["code"].as_str()),
        (400, Some("unknown_time_zone"))
    );
    let reply = call(
        &app,
        "GET",
        "/api/time/analysis/activity/timeline?day=soon&tz=UTC",
        None,
    )
    .await;
    assert_eq!(
        (reply.status, reply.json["code"].as_str()),
        (400, Some("invalid_date"))
    );
    db.drop().await;
}

#[tokio::test]
async fn a_row_that_began_days_before_the_period_still_counts_in_it() {
    let db = time_db("act_long_row").await;
    // Many short rows long before the Period, so a lower bound has real work to do.
    for day in 1..=5 {
        store(
            &db,
            "Code",
            "x — a",
            &format!("2026-04-0{day}T09:00:00Z"),
            &format!("2026-04-0{day}T09:10:00Z"),
        )
        .await;
    }
    // Began the 20th, ended the 26th: 6 days before the Period starts.
    store(
        &db,
        "Spotify",
        "",
        "2026-04-20T00:00:00Z",
        "2026-04-26T00:00:00Z",
    )
    .await;
    let app = time_app(&db, &FakeToggl::new());
    let totals = part(&app, "from=2026-04-24&to=2026-04-25&tz=UTC", "totals").await;
    let apps = totals["apps"].as_array().unwrap();
    assert_eq!(apps.len(), 1);
    assert_eq!(seconds_of(find(apps, "Spotify")), 2 * 24 * 3600);
    db.drop().await;
}

#[tokio::test]
async fn a_400_day_period_of_dense_activity_answers_every_route_quickly() {
    let db = time_db("act_perf").await;
    // 400 Days x 150 rows, 8 minutes each, across 40 apps and 300 titles.
    sqlx::query(
        "INSERT INTO time.activities (toggl_id, app, title, start_at, end_at, idle, raw)
         SELECT gen_random_uuid(), 'App' || (i % 40), 'Title ' || (i % 300),
                '2025-01-01T00:00:00Z'::timestamptz + d * interval '1 day' + i * interval '9 minutes',
                '2025-01-01T00:00:00Z'::timestamptz + d * interval '1 day' + i * interval '9 minutes' + interval '8 minutes',
                false, '{}'
         FROM generate_series(0, 399) d, generate_series(0, 149) i",
    )
    .execute(&db.pool)
    .await
    .unwrap();
    let app = time_app(&db, &FakeToggl::new());
    let q = "from=2025-01-01&to=2026-02-04&tz=Europe/London";
    // The 500 ms goal is for the release server; an unoptimised test build is several times slower.
    let budget_ms = if cfg!(debug_assertions) { 2500 } else { 500 };
    let started = std::time::Instant::now();
    let (status, _) = get(&app, &format!("/api/time/analysis/activity/overview?{q}")).await;
    let elapsed = started.elapsed();
    eprintln!("perf overview: {elapsed:?}");
    assert_eq!(status, 200);
    assert!(elapsed.as_millis() < budget_ms, "overview took {elapsed:?}");
    db.drop().await;
}
