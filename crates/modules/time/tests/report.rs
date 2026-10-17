//! The Report and facets routes.

mod common;

use common::{FakeToggl, call, time_app, time_db};
use vitarchy_core::test_db::TestDb;

/// Two clients, three projects (one archived), and entries on 2026-10-12..14 (UTC).
async fn seed(db: &TestDb) {
    let statements = [
        "INSERT INTO time.clients (toggl_id, workspace_id, name, archived, raw) VALUES
           (1, 7, 'Acme', false, '{}'), (2, 7, 'Old Co', true, '{}')",
        "INSERT INTO time.projects (toggl_id, workspace_id, client_id, name, active, color, raw) VALUES
           (10, 7, 1, 'Office', true, '#06aaf5', '{}'),
           (11, 7, NULL, 'Gym', true, NULL, '{}'),
           (12, 7, 2, 'Archived', false, NULL, '{}'),
           (13, 7, NULL, 'Never used', true, NULL, '{}')",
        "INSERT INTO time.time_entries
           (toggl_id, workspace_id, project_id, description, tags, start, stop, duration_s, raw) VALUES
           (1, 7, 10, 'Office',  '{focus}',         '2026-10-12T08:00Z', '2026-10-12T11:00Z', 10800, '{}'),
           (2, 7, 10, 'office ', '{focus,meeting}', '2026-10-12T13:00Z', '2026-10-12T17:00Z', 14400, '{}'),
           (3, 7, 11, 'Gym',     '{}',              '2026-10-13T18:00Z', '2026-10-13T19:00Z', 3600,  '{}'),
           (4, 7, 12, 'Old',     '{}',              '2026-10-14T23:00Z', '2026-10-15T01:00Z', 7200,  '{}'),
           (5, 7, 10, 'Office',  '{}',              '2026-10-05T08:00Z', '2026-10-05T09:00Z', 3600,  '{}')",
    ];
    for sql in statements {
        sqlx::query(sql).execute(&db.pool).await.unwrap();
    }
}

const WEEK: &str = "kind=week&from=2026-10-12&to=2026-10-18&tz=UTC";

#[tokio::test]
async fn a_report_groups_by_description_and_compares_with_the_previous_period() {
    let db = time_db("report_basic").await;
    seed(&db).await;
    let app = time_app(&db, &FakeToggl::new());

    let reply = call(&app, "GET", &format!("/api/time/report?{WEEK}"), None).await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    let json = &reply.json;
    assert_eq!(json["overview"]["total_s"], 10800 + 14400 + 3600 + 7200);
    assert_eq!(json["period"]["previous_from"], "2026-10-05");
    let office = &json["groups"][0];
    assert_eq!(office["label"], "Office");
    assert_eq!(office["stats"]["entries"], 2);
    assert_eq!(office["stats"]["days"], 1);
    assert_eq!(office["stats"]["per_day"]["avg_s"], 25200);
    assert_eq!(office["stats"]["per_entry"]["min_s"], 10800);
    assert_eq!(office["vs_previous"]["change"]["total_s"], 25200 - 3600);
    assert_eq!(office["entries"].as_array().unwrap().len(), 2);
    assert_eq!(json["daily"].as_array().unwrap().len(), 7);
    let gym = json["groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["label"] == "Gym");
    assert_eq!(gym.unwrap()["vs_previous"]["is_new"], true);
    db.drop().await;
}

#[tokio::test]
async fn a_report_carries_both_comparisons() {
    let db = time_db("report_usual").await;
    seed(&db).await;
    let app = time_app(&db, &FakeToggl::new());

    // The Usual's range is 2026-09-14..2026-10-11: only entry 5 (1h on 10-05) is in it.
    let reply = call(&app, "GET", &format!("/api/time/report?{WEEK}"), None).await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    let json = &reply.json;
    assert_eq!(json["period"]["usual_from"], "2026-09-14");
    assert_eq!(json["period"]["usual_to"], "2026-10-11");
    assert_eq!(json["overview"]["usual_days"], 1);
    // The earliest entry is on 2026-10-05, so 7 Days of the Usual's range exist. The test
    // runs on the real clock, so the week's elapsed Days are read back, not assumed.
    assert_eq!(json["period"]["usual_available_days"], 7);
    let elapsed = json["period"]["elapsed_days"].as_i64().unwrap();
    assert!((0..=7).contains(&elapsed));
    let scaled = (3600 * elapsed + 3) / 7;
    assert_eq!(json["overview"]["vs_usual"]["total_s"], 36000 - scaled);
    assert_eq!(json["overview"]["vs_previous"]["total_s"], 36000 - 3600);
    let groups = json["groups"].as_array().unwrap();
    let office = groups.iter().find(|g| g["label"] == "Office").unwrap();
    assert_eq!(office["vs_usual"]["is_new"], false);
    assert_eq!(office["vs_usual"]["change"]["total_s"], 25200 - scaled);
    assert_eq!(office["vs_usual"]["usual"]["days"], 1);
    assert_eq!(office["vs_usual"]["usual"]["per_day"]["avg_s"], 3600);
    let gym = groups.iter().find(|g| g["label"] == "Gym").unwrap();
    assert_eq!(gym["vs_usual"]["is_new"], true);
    assert!(gym["vs_usual"]["change"].is_null());
    db.drop().await;
}

#[tokio::test]
async fn filters_and_a_second_grouping_apply() {
    let db = time_db("report_filter").await;
    seed(&db).await;
    let app = time_app(&db, &FakeToggl::new());

    let uri = format!(
        "/api/time/report?{WEEK}&project=10&project=11&tag=focus&group1=project&group2=tag"
    );
    let reply = call(&app, "GET", &uri, None).await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    let groups = reply.json["groups"].as_array().unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0]["label"], "Office");
    // Two entries, one tagged focus+meeting: it counts under both, once in the overview.
    assert_eq!(groups[0]["children"].as_array().unwrap().len(), 2);
    assert_eq!(reply.json["overview"]["entries"], 2);

    let by_client = call(
        &app,
        "GET",
        &format!("/api/time/report?{WEEK}&client=2"),
        None,
    )
    .await;
    assert_eq!(by_client.json["groups"][0]["label"], "Old");
    assert_eq!(by_client.json["groups"][0]["stats"]["total_s"], 7200);
    assert_eq!(
        by_client.json["groups"][0]["stats"]["days"], 2,
        "midnight split"
    );
    db.drop().await;
}

#[tokio::test]
async fn bad_requests_are_400_problems() {
    let db = time_db("report_invalid").await;
    let app = time_app(&db, &FakeToggl::new());
    let bad = [
        "kind=week&from=2026-10-18&to=2026-10-12&tz=UTC",
        "kind=custom&from=2024-01-01&to=2026-01-01&tz=UTC",
        "kind=week&from=2026-10-12&to=2026-10-18&tz=Mars/Base",
        "kind=day&from=2026-10-12&to=2026-10-13&tz=UTC",
        "kind=week&from=2026-10-12&to=2026-10-18&tz=UTC&group1=tag&group2=tag",
        "kind=week&from=2026-10-12&to=2026-10-18&tz=UTC&project=abc",
        "kind=week&from=2026-10-12&tz=UTC",
    ];
    for query in bad {
        let reply = call(&app, "GET", &format!("/api/time/report?{query}"), None).await;
        assert_eq!(reply.status, 400, "{query}: {}", reply.text);
        assert_eq!(reply.content_type, "application/problem+json");
    }
    db.drop().await;
}

#[tokio::test]
async fn facets_list_only_what_has_entries_newest_first() {
    let db = time_db("report_facets").await;
    seed(&db).await;
    let app = time_app(&db, &FakeToggl::new());

    let reply = call(&app, "GET", "/api/time/facets", None).await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    let names = |list: &str, key: &str| -> Vec<String> {
        reply.json[list]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v[key].as_str().unwrap().to_owned())
            .collect()
    };
    assert_eq!(names("projects", "name"), ["Archived", "Gym", "Office"]);
    assert_eq!(reply.json["projects"][0]["active"], false);
    assert_eq!(reply.json["projects"][2]["client_id"], 1);
    assert_eq!(names("tags", "name"), ["focus", "meeting"]);
    assert_eq!(names("clients", "name"), ["Old Co", "Acme"]);
    assert_eq!(reply.json["clients"][0]["archived"], true);
    db.drop().await;
}

fn daily_seconds(json: &serde_json::Value) -> Vec<i64> {
    json["daily"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["seconds"].as_i64().unwrap())
        .collect()
}

#[tokio::test]
async fn the_time_zone_decides_which_day_an_entry_lands_on() {
    let db = time_db("report_zone_days").await;
    seed(&db).await;
    let app = time_app(&db, &FakeToggl::new());
    let week = |tz: &str| {
        format!("/api/time/report?kind=week&from=2026-10-12&to=2026-10-18&tz={tz}&client=2")
    };

    // 23:00Z-01:00Z crosses midnight in UTC: two Days, one hour each.
    let utc = call(&app, "GET", &week("UTC"), None).await;
    assert_eq!(daily_seconds(&utc.json), [0, 0, 3600, 3600, 0, 0, 0]);
    assert_eq!(utc.json["groups"][0]["stats"]["days"], 2);

    // 04:30-06:30 on the 15th in Kolkata: one Day, whole.
    let kolkata = call(&app, "GET", &week("Asia/Kolkata"), None).await;
    assert_eq!(kolkata.status, 200, "{}", kolkata.text);
    assert_eq!(daily_seconds(&kolkata.json), [0, 0, 0, 7200, 0, 0, 0]);
    assert_eq!(kolkata.json["groups"][0]["stats"]["days"], 1);

    // 16:00-18:00 on the 14th in Los Angeles (UTC-7).
    let la = call(&app, "GET", &week("America%2FLos_Angeles"), None).await;
    assert_eq!(la.status, 200, "{}", la.text);
    assert_eq!(daily_seconds(&la.json), [0, 0, 7200, 0, 0, 0, 0]);
    assert_eq!(la.json["groups"][0]["stats"]["days"], 1);
    db.drop().await;
}

#[tokio::test]
async fn the_time_zone_moves_which_entries_the_period_reads() {
    let db = time_db("report_zone_range").await;
    // 19:00Z on the 11th: still the 11th in UTC, already 00:30 on the 12th in Kolkata.
    sqlx::query(
        "INSERT INTO time.time_entries
           (toggl_id, workspace_id, project_id, description, tags, start, stop, duration_s, raw)
         VALUES (1, 7, NULL, 'Edge', '{}', '2026-10-11T19:00Z', '2026-10-11T20:00Z', 3600, '{}')",
    )
    .execute(&db.pool)
    .await
    .unwrap();
    let app = time_app(&db, &FakeToggl::new());
    let total =
        |tz: &str| format!("/api/time/report?kind=week&from=2026-10-12&to=2026-10-18&tz={tz}");

    let utc = call(&app, "GET", &total("UTC"), None).await;
    assert_eq!(utc.json["overview"]["total_s"], 0);
    // ... and it is the Previous Period's in UTC.
    assert_eq!(utc.json["overview"]["vs_previous"]["total_s"], -3600);

    let kolkata = call(&app, "GET", &total("Asia%2FKolkata"), None).await;
    assert_eq!(kolkata.json["overview"]["total_s"], 3600);
    assert_eq!(daily_seconds(&kolkata.json), [3600, 0, 0, 0, 0, 0, 0]);
    assert_eq!(kolkata.json["overview"]["vs_previous"]["total_s"], 3600);
    db.drop().await;
}

#[tokio::test]
async fn an_entry_across_the_boundary_is_counted_once_in_each_period() {
    let db = time_db("report_boundary").await;
    // 22:00Z on the 11th to 03:00Z on the 12th: 2h before the boundary, 3h after.
    sqlx::query(
        "INSERT INTO time.time_entries
           (toggl_id, workspace_id, project_id, description, tags, start, stop, duration_s, raw)
         VALUES (1, 7, NULL, 'Spans', '{}', '2026-10-11T22:00Z', '2026-10-12T03:00Z', 18000, '{}')",
    )
    .execute(&db.pool)
    .await
    .unwrap();
    let app = time_app(&db, &FakeToggl::new());

    let reply = call(&app, "GET", &format!("/api/time/report?{WEEK}"), None).await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    assert_eq!(reply.json["overview"]["total_s"], 3 * 3600);
    assert_eq!(
        reply.json["groups"][0]["entries"][0]["duration_s"],
        3 * 3600
    );
    assert_eq!(
        reply.json["groups"][0]["vs_previous"]["change"]["total_s"],
        3600
    );
    db.drop().await;
}

#[tokio::test]
async fn a_blank_search_and_empty_filter_values_are_ignored() {
    let db = time_db("report_blank_filters").await;
    seed(&db).await;
    let app = time_app(&db, &FakeToggl::new());
    let plain = call(&app, "GET", &format!("/api/time/report?{WEEK}"), None).await;

    let blank = call(
        &app,
        "GET",
        &format!("/api/time/report?{WEEK}&q=%20%20&tag=&project="),
        None,
    )
    .await;

    assert_eq!(blank.status, 200, "{}", blank.text);
    assert_eq!(blank.json["overview"], plain.json["overview"]);
    let negative = call(
        &app,
        "GET",
        &format!("/api/time/report?{WEEK}&project=-1"),
        None,
    )
    .await;
    assert_eq!(negative.status, 400);
    db.drop().await;
}
