//! `/api/time/analysis/lifelog/...`: the Life log overview and timeline over stored Time Entries.

mod common;

use axum::Router;
use common::{FakeToggl, call, time_app, time_db};
use serde_json::Value;
use vitarchy_core::test_db::TestDb;

const HOUR: i64 = 3600;

async fn project(db: &TestDb, id: i64, name: &str, color: Option<&str>) {
    sqlx::query("INSERT INTO time.projects (toggl_id, workspace_id, name, color, raw) VALUES ($1, 7, $2, $3, '{}')")
        .bind(id)
        .bind(name)
        .bind(color)
        .execute(&db.pool)
        .await
        .unwrap();
}

/// Stores one synthetic entry (times are RFC 3339 text).
async fn store(db: &TestDb, project_id: Option<i64>, description: &str, start: &str, stop: &str) {
    sqlx::query(
        "INSERT INTO time.time_entries (toggl_id, workspace_id, project_id, description, start, stop, duration_s, raw)
         VALUES ((random() * 1e12)::bigint, 7, $1, $2, $3::timestamptz, $4::timestamptz,
                 extract(epoch FROM $4::timestamptz - $3::timestamptz)::bigint, '{}')",
    )
    .bind(project_id)
    .bind(description)
    .bind(start)
    .bind(stop)
    .execute(&db.pool)
    .await
    .unwrap();
}

async fn get(app: &Router, uri: &str) -> (u16, Value) {
    let reply = call(app, "GET", uri, None).await;
    (reply.status, reply.json)
}

async fn overview(app: &Router, query: &str) -> Value {
    let (status, body) = get(app, &format!("/api/time/analysis/lifelog/overview?{query}")).await;
    assert_eq!(status, 200, "{body}");
    body
}

const UTC_WEEK: &str = "from=2026-05-04&to=2026-05-10&tz=UTC";
const OFFICE: i64 = 1;
const SLEEP: i64 = 2;
const BODY: i64 = 3;

/// A small life: an office day, a workout inside it, and a night's sleep over midnight.
async fn seeded(label: &str) -> (TestDb, Router) {
    let db = time_db(label).await;
    project(&db, OFFICE, "office", Some("#2a78d6")).await;
    project(&db, SLEEP, "sleep", Some("#6250d6")).await;
    project(&db, BODY, "body", None).await;
    store(
        &db,
        Some(OFFICE),
        "inside_office",
        "2026-05-04T09:00:00Z",
        "2026-05-04T17:00:00Z",
    )
    .await;
    store(
        &db,
        Some(BODY),
        "workout",
        "2026-05-04T16:00:00Z",
        "2026-05-04T17:30:00Z",
    )
    .await;
    store(
        &db,
        Some(SLEEP),
        "sleep",
        "2026-05-04T23:00:00Z",
        "2026-05-05T07:00:00Z",
    )
    .await;
    store(
        &db,
        None,
        "",
        "2026-05-06T12:00:00Z",
        "2026-05-06T13:00:00Z",
    )
    .await;
    let app = time_app(&db, &FakeToggl::new());
    (db, app)
}

fn day<'a>(body: &'a Value, date: &str) -> &'a Value {
    body["split"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["date"] == date)
        .unwrap()
}

fn seconds_for(day: &Value, project: &str) -> i64 {
    day["by_project"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["project"] == project)
        .map_or(0, |p| p["seconds"].as_i64().unwrap())
}

#[tokio::test]
async fn the_split_divides_each_day_by_project_and_cuts_sleep_at_midnight() {
    let (db, app) = seeded("ll_split").await;
    let body = overview(&app, UTC_WEEK).await;
    assert_eq!(body["split"].as_array().unwrap().len(), 7);
    let monday = day(&body, "2026-05-04");
    assert_eq!(
        (
            seconds_for(monday, "office"),
            seconds_for(monday, "body"),
            seconds_for(monday, "sleep")
        ),
        (8 * HOUR, 90 * 60, HOUR)
    );
    let tuesday = day(&body, "2026-05-05");
    assert_eq!(seconds_for(tuesday, "sleep"), 7 * HOUR);
    assert_eq!(seconds_for(day(&body, "2026-05-06"), "No project"), HOUR);
    db.drop().await;
}

#[tokio::test]
async fn unlogged_is_the_day_minus_the_union_so_overlap_is_not_counted_twice() {
    let (db, app) = seeded("ll_unlogged").await;
    let body = overview(&app, UTC_WEEK).await;
    let monday = day(&body, "2026-05-04");
    // Office 09-17 and workout 16-17:30 overlap: the union is 09:00-17:30, plus 23:00-24:00 of sleep.
    assert_eq!(monday["elapsed_seconds"], 24 * HOUR);
    assert_eq!(
        monday["unlogged_seconds"],
        24 * HOUR - (8 * HOUR + 30 * 60) - HOUR
    );
    // Project time still counts each entry in full: 8 h + 1.5 h + 1 h.
    let projects: i64 = monday["by_project"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["seconds"].as_i64().unwrap())
        .sum();
    assert_eq!(projects, 10 * HOUR + 30 * 60);
    assert_eq!(day(&body, "2026-05-07")["unlogged_seconds"], 24 * HOUR);
    db.drop().await;
}

#[tokio::test]
async fn totals_rank_projects_with_descriptions_colours_entries_and_days() {
    let (db, app) = seeded("ll_totals").await;
    let totals = &overview(&app, UTC_WEEK).await["totals"];
    assert_eq!(
        totals["logged_seconds"],
        (8 * HOUR + 90 * 60 + 8 * HOUR + HOUR)
    );
    let projects = totals["by_project"].as_array().unwrap();
    assert_eq!(projects[0]["name"], "office");
    assert_eq!(projects[0]["color"], "#2a78d6");
    assert_eq!(
        projects[0]["descriptions"][0]["description"],
        "inside_office"
    );
    let sleep = projects.iter().find(|p| p["name"] == "sleep").unwrap();
    assert_eq!(
        (
            sleep["entries"].as_i64(),
            sleep["days"].as_i64(),
            sleep["seconds"].as_i64()
        ),
        (Some(1), Some(2), Some(8 * HOUR))
    );
    let body = projects.iter().find(|p| p["name"] == "body").unwrap();
    assert!(body["color"].is_null());
    assert!(projects.iter().any(|p| p["name"] == "No project"));
    assert!(
        totals["by_description"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["description"] == "No description")
    );
    db.drop().await;
}

#[tokio::test]
async fn totals_follow_the_requested_time_zone() {
    let (db, app) = seeded("ll_tz").await;
    // Sleep runs 04:30-12:30 on the 5th in Kolkata: all on the 5th there, split in UTC.
    let kolkata = overview(&app, "from=2026-05-05&to=2026-05-05&tz=Asia/Kolkata").await;
    assert_eq!(seconds_for(day(&kolkata, "2026-05-05"), "sleep"), 8 * HOUR);
    let utc = overview(&app, "from=2026-05-05&to=2026-05-05&tz=UTC").await;
    assert_eq!(seconds_for(day(&utc, "2026-05-05"), "sleep"), 7 * HOUR);
    db.drop().await;
}

#[tokio::test]
async fn an_empty_period_is_empty_not_an_error() {
    let (db, app) = seeded("ll_empty").await;
    let body = overview(&app, "from=2020-01-01&to=2020-01-03&tz=UTC").await;
    assert_eq!(body["totals"]["logged_seconds"], 0);
    assert!(body["totals"]["by_project"].as_array().unwrap().is_empty());
    assert!(body["description_stats"].as_array().unwrap().is_empty());
    assert_eq!(body["split"].as_array().unwrap().len(), 3);
    assert_eq!(body["split"][0]["unlogged_seconds"], 24 * HOUR);
    let (_, lane) = get(
        &app,
        "/api/time/analysis/lifelog/timeline?day=2020-01-01&tz=UTC",
    )
    .await;
    assert!(lane["spans"].as_array().unwrap().is_empty());
    db.drop().await;
}

#[tokio::test]
async fn the_present_day_is_cut_at_now_and_later_days_have_nothing_unlogged() {
    let db = time_db("ll_now").await;
    let app = time_app(&db, &FakeToggl::new());
    let today = chrono::Utc::now().date_naive();
    let tomorrow = today + chrono::Days::new(1);
    let body = overview(&app, &format!("from={today}&to={tomorrow}&tz=UTC")).await;
    let (now_day, later_day) = (&body["split"][0], &body["split"][1]);
    let so_far = (chrono::Utc::now() - today.and_hms_opt(0, 0, 0).unwrap().and_utc()).num_seconds();
    let elapsed = now_day["elapsed_seconds"].as_i64().unwrap();
    assert!(
        (0..=so_far + 1).contains(&elapsed) && elapsed >= so_far - 60,
        "{elapsed} vs {so_far}"
    );
    assert_eq!(now_day["unlogged_seconds"], now_day["elapsed_seconds"]);
    assert_eq!(
        (
            later_day["elapsed_seconds"].as_i64(),
            later_day["unlogged_seconds"].as_i64()
        ),
        (Some(0), Some(0))
    );
    db.drop().await;
}

#[tokio::test]
async fn description_stats_keep_sleep_around_midnight_together() {
    let db = time_db("ll_stats").await;
    project(&db, SLEEP, "sleep", None).await;
    // Six nights starting 23:30, 23:50, 00:10, 00:30, 01:00 and 23:40 local (Kolkata is UTC+5:30).
    for (start, stop) in [
        ("2026-05-04T18:00:00Z", "2026-05-05T02:00:00Z"),
        ("2026-05-05T18:20:00Z", "2026-05-06T02:00:00Z"),
        ("2026-05-06T18:40:00Z", "2026-05-07T02:00:00Z"),
        ("2026-05-07T19:00:00Z", "2026-05-08T02:00:00Z"),
        ("2026-05-08T19:30:00Z", "2026-05-09T02:00:00Z"),
        ("2026-05-10T18:10:00Z", "2026-05-11T02:00:00Z"),
    ] {
        store(&db, Some(SLEEP), "sleep", start, stop).await;
    }
    store(
        &db,
        Some(SLEEP),
        "nap",
        "2026-05-04T08:00:00Z",
        "2026-05-04T08:20:00Z",
    )
    .await;
    let app = time_app(&db, &FakeToggl::new());
    let body = overview(&app, "from=2026-05-04&to=2026-05-11&tz=Asia/Kolkata").await;
    let stats = body["description_stats"].as_array().unwrap();
    assert_eq!(stats.len(), 1, "nap is below the five-day bar");
    let sleep = &stats[0];
    assert_eq!(
        (
            sleep["description"].as_str(),
            sleep["days"].as_i64(),
            sleep["entries"].as_i64()
        ),
        (Some("sleep"), Some(6), Some(6))
    );
    let minute = |part: &str, key: &str| sleep[part][key].as_i64().unwrap();
    // Median of 23:30 23:40 23:50 00:10 00:30 01:00 is 00:00 (midnight), not midday.
    assert_eq!(minute("start", "median_minute"), 0);
    assert!((1420..1430).contains(&minute("start", "p25_minute")));
    assert_eq!(minute("start", "p75_minute"), 25);
    assert_eq!(minute("end", "median_minute"), 7 * 60 + 30);
    // Nights of 8 h, 7 h 40, 7 h 20, 7 h, 6 h 30 and 7 h 50: the middle two are 7 h 20 and 7 h 40.
    assert_eq!(sleep["duration"]["median_seconds"], 7 * HOUR + 30 * 60);
    db.drop().await;
}

#[tokio::test]
async fn the_timeline_clips_entries_to_the_day_and_carries_colour_and_description() {
    let (db, app) = seeded("ll_lane").await;
    let (status, lane) = get(
        &app,
        "/api/time/analysis/lifelog/timeline?day=2026-05-05&tz=UTC",
    )
    .await;
    assert_eq!(status, 200);
    let spans = lane["spans"].as_array().unwrap();
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0]["start"], "2026-05-05T00:00:00Z");
    assert_eq!(spans[0]["end"], "2026-05-05T07:00:00Z");
    assert_eq!(
        (
            spans[0]["project"].as_str(),
            spans[0]["color"].as_str(),
            spans[0]["description"].as_str()
        ),
        (Some("sleep"), Some("#6250d6"), Some("sleep"))
    );
    assert_eq!(lane["day_start"], "2026-05-05T00:00:00Z");
    let (_, monday) = get(
        &app,
        "/api/time/analysis/lifelog/timeline?day=2026-05-04&tz=UTC",
    )
    .await;
    assert_eq!(monday["spans"].as_array().unwrap().len(), 3);
    db.drop().await;
}

#[tokio::test]
async fn an_entry_that_began_days_before_the_period_still_counts_in_it() {
    let db = time_db("ll_long").await;
    project(&db, OFFICE, "office", None).await;
    for day in 1..=5 {
        store(
            &db,
            Some(OFFICE),
            "job",
            &format!("2026-04-0{day}T09:00:00Z"),
            &format!("2026-04-0{day}T10:00:00Z"),
        )
        .await;
    }
    store(
        &db,
        Some(OFFICE),
        "retreat",
        "2026-04-20T00:00:00Z",
        "2026-04-26T00:00:00Z",
    )
    .await;
    let app = time_app(&db, &FakeToggl::new());
    let body = overview(&app, "from=2026-04-24&to=2026-04-25&tz=UTC").await;
    assert_eq!(body["totals"]["logged_seconds"], 48 * HOUR);
    db.drop().await;
}

#[tokio::test]
async fn bad_parameters_are_coded_400s_on_every_route() {
    let (db, app) = seeded("ll_bad").await;
    let cases = [
        (
            "/api/time/analysis/lifelog/overview?from=2026-05-04&to=2026-05-10&tz=Nowhere/Land",
            "unknown_time_zone",
        ),
        (
            "/api/time/analysis/lifelog/overview?from=2026-05-10&to=2026-05-04&tz=UTC",
            "period_reversed",
        ),
        (
            "/api/time/analysis/lifelog/overview?from=x&to=2026-05-04&tz=UTC",
            "invalid_date",
        ),
        (
            "/api/time/analysis/lifelog/overview?from=2024-01-01&to=2026-05-04&tz=UTC",
            "period_too_long",
        ),
        (
            "/api/time/analysis/lifelog/timeline?day=soon&tz=UTC",
            "invalid_date",
        ),
        (
            "/api/time/analysis/lifelog/timeline?day=2026-05-04",
            "unknown_time_zone",
        ),
    ];
    for (uri, code) in cases {
        let reply = call(&app, "GET", uri, None).await;
        assert_eq!(
            (reply.status, reply.json["code"].as_str()),
            (400, Some(code)),
            "{uri}"
        );
    }
    db.drop().await;
}

#[tokio::test]
async fn a_400_day_period_of_dense_entries_answers_quickly() {
    let db = time_db("ll_perf").await;
    project(&db, OFFICE, "office", Some("#2a78d6")).await;
    sqlx::query(
        "INSERT INTO time.time_entries (toggl_id, workspace_id, project_id, description, start, stop, duration_s, raw)
         SELECT d * 100 + i, 7, 1, 'task ' || (i % 12),
                '2025-01-01T00:00:00Z'::timestamptz + d * interval '1 day' + i * interval '100 minutes',
                '2025-01-01T00:00:00Z'::timestamptz + d * interval '1 day' + i * interval '100 minutes' + interval '60 minutes',
                3600, '{}'
         FROM generate_series(0, 399) d, generate_series(0, 12) i",
    )
    .execute(&db.pool)
    .await
    .unwrap();
    let app = time_app(&db, &FakeToggl::new());
    let budget_ms = if cfg!(debug_assertions) { 2500 } else { 500 };
    let started = std::time::Instant::now();
    let (status, _) = get(
        &app,
        "/api/time/analysis/lifelog/overview?from=2025-01-01&to=2026-02-04&tz=Europe/London",
    )
    .await;
    let elapsed = started.elapsed();
    eprintln!("perf lifelog overview: {elapsed:?}");
    assert_eq!(status, 200);
    assert!(elapsed.as_millis() < budget_ms, "overview took {elapsed:?}");
    db.drop().await;
}
