//! `/api/time/routines...`: suggestions, saved Routines and their Occurrences over stored Time Entries.

mod common;

use axum::Router;
use common::{FakeToggl, call, time_app, time_db};
use serde_json::{Value, json};
use vitarchy_core::test_db::TestDb;

const UTC_MONTH: &str = "from=2026-05-01&to=2026-05-31&tz=UTC";

async fn store(db: &TestDb, project: Option<&str>, description: &str, start: &str, stop: &str) {
    if let Some(name) = project {
        sqlx::query("INSERT INTO time.projects (toggl_id, workspace_id, name, raw) VALUES (hashtext($1), 7, $1, '{}') ON CONFLICT DO NOTHING")
            .bind(name)
            .execute(&db.pool)
            .await
            .unwrap();
    }
    sqlx::query(
        "INSERT INTO time.time_entries (toggl_id, workspace_id, project_id, description, start, stop, duration_s, raw)
         VALUES ((random() * 1e12)::bigint, 7, hashtext($1), $2, $3::timestamptz, $4::timestamptz,
                 extract(epoch FROM $4::timestamptz - $3::timestamptz)::bigint, '{}')",
    )
    .bind(project)
    .bind(description)
    .bind(start)
    .bind(stop)
    .execute(&db.pool)
    .await
    .unwrap();
}

/// Office days on each of `days` (May 2026): commute in, office, commute home.
async fn office_days(db: &TestDb, days: &[u32]) {
    for d in days {
        let at = |t: &str| format!("2026-05-{d:02}T{t}:00Z");
        store(
            db,
            Some("office"),
            "travel to_office",
            &at("08:00"),
            &at("09:00"),
        )
        .await;
        store(
            db,
            Some("office"),
            "inside_office",
            &at("09:15"),
            &at("17:00"),
        )
        .await;
        store(
            db,
            Some("office"),
            "travel from_office",
            &at("17:10"),
            &at("18:00"),
        )
        .await;
    }
}

fn office_steps() -> Value {
    json!([
        {"label": "To office", "kind": "description", "value": "travel to_office"},
        {"label": "At office", "kind": "description", "value": "inside_office"},
        {"label": "Home", "kind": "description", "value": "travel from_office"},
    ])
}

fn batch_uri() -> String {
    format!("/api/time/routines/occurrences?{UTC_MONTH}")
}

async fn app(label: &str) -> (TestDb, Router) {
    let db = time_db(label).await;
    let app = time_app(&db, &FakeToggl::new());
    (db, app)
}

async fn request(app: &Router, method: &str, uri: &str, body: Option<Value>) -> (u16, Value) {
    let text = body.map(|b| b.to_string());
    let reply = call(app, method, uri, text.as_deref()).await;
    (reply.status, reply.json)
}

async fn create(app: &Router, name: &str, steps: Value) -> (u16, Value) {
    request(
        app,
        "POST",
        "/api/time/routines",
        Some(json!({"name": name, "steps": steps})),
    )
    .await
}

#[tokio::test]
async fn suggestions_offer_the_recurring_chain_and_ignore_sleep_over_midnight() {
    let (db, app) = app("routines_suggest").await;
    office_days(&db, &[4, 5, 6, 7, 8, 11, 12]).await;
    for d in 4..=10 {
        let at = |day: u32, t: &str| format!("2026-05-{day:02}T{t}:00Z");
        store(
            &db,
            Some("sleep"),
            "sleep",
            &at(d, "23:00"),
            &at(d + 1, "07:00"),
        )
        .await;
        store(
            &db,
            Some("body"),
            "bath",
            &at(d + 1, "07:30"),
            &at(d + 1, "07:45"),
        )
        .await;
    }
    let (status, body) = request(
        &app,
        "GET",
        &format!("/api/time/routines/suggestions?{UTC_MONTH}"),
        None,
    )
    .await;
    assert_eq!(status, 200, "{body}");
    let found = body["suggestions"].as_array().unwrap();
    let office = found
        .iter()
        .find(|s| s["steps"][0] == "travel to_office")
        .unwrap();
    assert_eq!(
        office["steps"],
        json!(["travel to_office", "inside_office", "travel from_office"])
    );
    assert_eq!(office["days"], 7);
    assert!(
        !found.iter().any(|s| s["steps"] == json!(["sleep", "bath"])),
        "sleep must not link to the next Day's bath: {found:?}"
    );
    db.drop().await;
}

#[tokio::test]
async fn a_saved_routine_stops_being_suggested() {
    let (db, app) = app("routines_suggest_saved").await;
    office_days(&db, &[4, 5, 6, 7, 8]).await;
    let uri = format!("/api/time/routines/suggestions?{UTC_MONTH}");
    assert_eq!(
        request(&app, "GET", &uri, None).await.1["suggestions"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(create(&app, "Office", office_steps()).await.0, 201);
    assert!(
        request(&app, "GET", &uri, None).await.1["suggestions"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    db.drop().await;
}

#[tokio::test]
async fn analysis_routes_reject_a_bad_zone_with_a_code() {
    let (db, app) = app("routines_bad_zone").await;
    for uri in [
        "/api/time/routines/suggestions?from=2026-05-01&to=2026-05-02&tz=Nowhere",
        "/api/time/routines/occurrences?from=2026-05-01&to=2026-05-02&tz=Nowhere",
    ] {
        let (status, body) = request(&app, "GET", uri, None).await;
        assert_eq!(
            (status, body["code"].as_str()),
            (400, Some("unknown_time_zone")),
            "{uri}"
        );
    }
    db.drop().await;
}

#[tokio::test]
async fn routines_are_created_listed_replaced_and_deleted() {
    let (db, app) = app("routines_crud").await;
    let (status, created) = create(&app, "  Office day ", office_steps()).await;
    assert_eq!(status, 201, "{created}");
    assert_eq!(created["name"], "Office day");
    assert_eq!(created["steps"][1]["value"], "inside_office");
    let id = created["id"].as_i64().unwrap();

    let (_, listed) = request(&app, "GET", "/api/time/routines", None).await;
    assert_eq!(listed["routines"].as_array().unwrap().len(), 1);

    let new_steps = json!([{"label": "Office", "kind": "project", "value": "office"}]);
    let (status, replaced) = request(
        &app,
        "PUT",
        &format!("/api/time/routines/{id}"),
        Some(json!({"name": "Work", "steps": new_steps})),
    )
    .await;
    assert_eq!(status, 200, "{replaced}");
    assert_eq!(
        (
            replaced["name"].as_str(),
            replaced["steps"].as_array().unwrap().len()
        ),
        (Some("Work"), 1)
    );
    assert_eq!(replaced["steps"][0]["kind"], "project");

    let (status, _) = request(&app, "DELETE", &format!("/api/time/routines/{id}"), None).await;
    assert_eq!(status, 204);
    let steps_left: i64 = sqlx::query_scalar("SELECT count(*) FROM time.routine_steps")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(steps_left, 0, "deleting a Routine deletes its Steps");
    let (status, body) = request(&app, "DELETE", &format!("/api/time/routines/{id}"), None).await;
    assert_eq!(
        (status, body["code"].as_str()),
        (404, Some("routine_not_found"))
    );
    db.drop().await;
}

#[tokio::test]
async fn invalid_routines_are_coded_problems() {
    let (db, app) = app("routines_validation").await;
    let one = json!([{"label": "a", "kind": "description", "value": "b"}]);
    let nine: Vec<Value> = (0..9).map(|_| one[0].clone()).collect();
    let cases = [
        (json!({"name": " ", "steps": one}), "routine_name_empty"),
        (
            json!({"name": "x".repeat(81), "steps": one}),
            "routine_name_too_long",
        ),
        (json!({"name": "n", "steps": []}), "routine_no_steps"),
        (
            json!({"name": "n", "steps": nine}),
            "routine_too_many_steps",
        ),
        (
            json!({"name": "n", "steps": [{"label": " ", "kind": "description", "value": "b"}]}),
            "routine_label_empty",
        ),
        (
            json!({"name": "n", "steps": [{"label": "a", "kind": "project", "value": ""}]}),
            "routine_value_empty",
        ),
    ];
    for (body, code) in cases {
        let (status, reply) = request(&app, "POST", "/api/time/routines", Some(body)).await;
        assert_eq!(
            (status, reply["code"].as_str()),
            (400, Some(code)),
            "{reply}"
        );
    }
    let (status, _) = request(
        &app,
        "POST",
        "/api/time/routines",
        Some(json!({"name": "n", "steps": [{"label": "a", "kind": "tag", "value": "b"}]})),
    )
    .await;
    assert_eq!(status, 400, "an unknown match kind is a bad request");
    assert!(
        request(&app, "GET", "/api/time/routines", None).await.1["routines"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    db.drop().await;
}

#[tokio::test]
async fn names_are_unique_ignoring_case_and_missing_routines_are_404() {
    let (db, app) = app("routines_names").await;
    let (_, first) = create(&app, "Office", office_steps()).await;
    let (status, clash) = create(&app, "office", office_steps()).await;
    assert_eq!(
        (status, clash["code"].as_str()),
        (409, Some("routine_name_taken"))
    );
    let (_, second) = create(&app, "Gym", office_steps()).await;
    let (status, clash) = request(
        &app,
        "PUT",
        &format!("/api/time/routines/{}", second["id"]),
        Some(json!({"name": "OFFICE", "steps": office_steps()})),
    )
    .await;
    assert_eq!(
        (status, clash["code"].as_str()),
        (409, Some("routine_name_taken"))
    );
    // Keeping its own name while editing is fine.
    let (status, _) = request(
        &app,
        "PUT",
        &format!("/api/time/routines/{}", first["id"]),
        Some(json!({"name": "OFFICE", "steps": office_steps()})),
    )
    .await;
    assert_eq!(status, 200);
    let (status, body) = request(
        &app,
        "PUT",
        "/api/time/routines/99999",
        Some(json!({"name": "x", "steps": office_steps()})),
    )
    .await;
    assert_eq!(
        (status, body["code"].as_str()),
        (404, Some("routine_not_found"))
    );
    db.drop().await;
}

#[tokio::test]
async fn occurrences_split_complete_and_incomplete_days_with_stats() {
    let (db, app) = app("routines_occurrences").await;
    office_days(&db, &[4, 5, 6]).await;
    // Wednesday the 6th... a partial Day: no commute in on the 7th.
    store(
        &db,
        Some("office"),
        "inside_office",
        "2026-05-07T09:15:00Z",
        "2026-05-07T17:00:00Z",
    )
    .await;
    store(
        &db,
        Some("office"),
        "travel from_office",
        "2026-05-07T17:10:00Z",
        "2026-05-07T18:00:00Z",
    )
    .await;
    create(&app, "Office", office_steps()).await;
    let (status, all) = request(&app, "GET", &batch_uri(), None).await;
    assert_eq!(status, 200, "{all}");
    let body = &all["routines"][0];
    let days = body["occurrences"].as_array().unwrap();
    assert_eq!(days.len(), 4);
    assert_eq!(days[0]["date"], "2026-05-04");
    assert_eq!(days[0]["complete"], true);
    assert_eq!(days[0]["door_to_door_seconds"], 10 * 3600);
    assert_eq!(days[0]["gaps"], json!([900, 600]));
    assert_eq!(days[3]["complete"], false);
    assert!(days[3]["steps"][0].is_null() && !days[3]["steps"][1].is_null());
    assert!(days[3]["door_to_door_seconds"].is_null());
    let figures = &body["stats"];
    assert_eq!(
        (&figures["complete_days"], &figures["incomplete_days"]),
        (&json!(3), &json!(1))
    );
    assert_eq!(figures["door_to_door"]["median_seconds"], 10 * 3600);
    assert_eq!(figures["steps"][1]["days"], 4);
    assert_eq!(body["routine"]["name"], "Office");
    db.drop().await;
}

#[tokio::test]
async fn an_entry_belongs_to_the_day_it_starts_in_across_midnight() {
    let (db, app) = app("routines_midnight").await;
    for d in 4..=6 {
        let at = |day: u32, t: &str| format!("2026-05-{day:02}T{t}:00Z");
        store(&db, None, "dinner", &at(d, "21:00"), &at(d, "21:45")).await;
        store(&db, None, "sleep", &at(d, "23:00"), &at(d + 1, "07:00")).await;
    }
    let steps = json!([
        {"label": "Dinner", "kind": "description", "value": "Dinner"},
        {"label": "Sleep", "kind": "description", "value": " sleep "},
    ]);
    create(&app, "Evening", steps).await;
    let (_, all) = request(&app, "GET", &batch_uri(), None).await;
    let body = &all["routines"][0];
    let days = body["occurrences"].as_array().unwrap();
    assert_eq!(days.len(), 3);
    assert!(days.iter().all(|d| d["complete"] == true));
    assert_eq!(
        body["stats"]["steps"][1]["duration"]["median_seconds"],
        8 * 3600
    );
    db.drop().await;
}

#[tokio::test]
async fn a_step_logged_after_midnight_finishes_the_evening_even_at_the_edge_of_the_period() {
    let (db, app) = app("routines_spill").await;
    let at = |day: u32, t: &str| format!("2026-05-{day:02}T{t}:00Z");
    for d in 4..=6 {
        store(&db, None, "dinner", &at(d, "19:00"), &at(d, "20:00")).await;
        store(&db, None, "bath", &at(d, "22:30"), &at(d, "22:50")).await;
        store(&db, None, "sleep", &at(d + 1, "00:10"), &at(d + 1, "07:00")).await;
    }
    // A morning entry that starts after the cutoff is a new Day's, not the evening's.
    store(&db, None, "sleep", &at(8, "07:00"), &at(8, "08:00")).await;
    let steps = json!([
        {"label": "Dinner", "kind": "description", "value": "dinner"},
        {"label": "Bath", "kind": "description", "value": "bath"},
        {"label": "Sleep", "kind": "description", "value": "sleep"},
    ]);
    create(&app, "Evening", steps).await;
    // The Period ends on the 6th: its sleep starts on the 7th, outside the Period.
    let uri = "/api/time/routines/occurrences?from=2026-05-04&to=2026-05-06&tz=UTC";
    let (_, all) = request(&app, "GET", uri, None).await;
    let body = &all["routines"][0];
    assert_eq!(body["stats"]["complete_days"], 3, "{body}");
    assert_eq!(body["stats"]["incomplete_days"], 0);
    // The 7th's own view does not count the sleep that belongs to the evening of the 6th.
    let uri = "/api/time/routines/occurrences?from=2026-05-07&to=2026-05-08&tz=UTC";
    let (_, all) = request(&app, "GET", uri, None).await;
    let days = all["routines"][0]["occurrences"].as_array().unwrap();
    assert_eq!(days.len(), 1, "{days:?}");
    assert_eq!(days[0]["date"], "2026-05-08");
    db.drop().await;
}

#[tokio::test]
async fn a_routine_with_no_matching_days_has_empty_occurrences() {
    let (db, app) = app("routines_empty").await;
    create(&app, "Office", office_steps()).await;
    let (status, all) = request(&app, "GET", &batch_uri(), None).await;
    assert_eq!(status, 200);
    let body = &all["routines"][0];
    assert!(body["occurrences"].as_array().unwrap().is_empty());
    assert!(body["stats"]["door_to_door"].is_null());
    db.drop().await;
}

#[tokio::test]
async fn every_routine_comes_back_in_one_request_by_name() {
    let (db, app) = app("routines_batch").await;
    office_days(&db, &[4, 5]).await;
    create(&app, "Office", office_steps()).await;
    create(
        &app,
        "Bath",
        json!([{"label": "Bath", "kind": "description", "value": "bath"}]),
    )
    .await;
    let (status, all) = request(&app, "GET", &batch_uri(), None).await;
    assert_eq!(status, 200, "{all}");
    let routines = all["routines"].as_array().unwrap();
    let names: Vec<&str> = routines
        .iter()
        .map(|r| r["routine"]["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["Bath", "Office"]);
    assert_eq!(routines[0]["occurrences"].as_array().unwrap().len(), 0);
    assert_eq!(routines[1]["occurrences"].as_array().unwrap().len(), 2);
    assert_eq!(routines[1]["routine"]["steps"].as_array().unwrap().len(), 3);
    db.drop().await;
}

#[tokio::test]
async fn no_routines_is_an_empty_list() {
    let (db, app) = app("routines_batch_none").await;
    let (status, all) = request(&app, "GET", &batch_uri(), None).await;
    assert_eq!(
        (status, all["routines"].as_array().map(Vec::len)),
        (200, Some(0))
    );
    db.drop().await;
}

#[tokio::test]
async fn a_step_can_match_entries_that_have_no_description() {
    let (db, app) = app("routines_no_description").await;
    store(
        &db,
        None,
        "",
        "2026-05-04T09:00:00Z",
        "2026-05-04T10:00:00Z",
    )
    .await;
    create(
        &app,
        "Blank",
        json!([{"label": "Unnamed", "kind": "description", "value": "No description"}]),
    )
    .await;
    let (_, all) = request(&app, "GET", &batch_uri(), None).await;
    assert_eq!(
        all["routines"][0]["occurrences"].as_array().unwrap().len(),
        1
    );
    db.drop().await;
}
