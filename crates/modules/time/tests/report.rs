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
    assert_eq!(office["change"]["total_s"], 25200 - 3600);
    assert_eq!(office["entries"].as_array().unwrap().len(), 2);
    assert_eq!(json["daily"].as_array().unwrap().len(), 7);
    let gym = json["groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["label"] == "Gym");
    assert_eq!(gym.unwrap()["is_new"], true);
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
