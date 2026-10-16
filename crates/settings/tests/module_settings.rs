mod common;

use common::{call, settings_app, settings_db};
use serde_json::json;

#[tokio::test]
async fn module_settings_round_trip_under_their_id() {
    let db = settings_db("modules").await;
    let app = settings_app(&db);
    let body = json!({"goal_hours": 8, "show_streak": true});

    let put = call(
        &app,
        "PUT",
        "/api/settings/modules/time-tracker",
        Some(&body.to_string()),
    )
    .await;
    let get = call(&app, "GET", "/api/settings", None).await;

    assert_eq!(put.status, 200);
    assert_eq!(put.json, body);
    assert_eq!(get.json["modules"], json!({"time-tracker": body}));
    assert_eq!(get.json["core"]["theme"], "system");
    db.drop().await;
}

#[tokio::test]
async fn a_module_id_cannot_shadow_core_scope() {
    let db = settings_db("scopes").await;
    let app = settings_app(&db);

    call(
        &app,
        "PUT",
        "/api/settings/core",
        Some(r#"{"theme":"dark"}"#),
    )
    .await;
    call(
        &app,
        "PUT",
        "/api/settings/modules/time",
        Some(r#"{"a":1}"#),
    )
    .await;
    let get = call(&app, "GET", "/api/settings", None).await;

    assert_eq!(get.json["core"]["theme"], "dark");
    assert_eq!(get.json["modules"], json!({"time": {"a": 1}}));
    db.drop().await;
}

#[tokio::test]
async fn bad_module_id_or_non_object_body_is_a_400_problem() {
    let db = settings_db("badmodule").await;
    let app = settings_app(&db);
    let cases = [
        ("/api/settings/modules/core", r#"{"a":1}"#),
        ("/api/settings/modules/Time", r#"{"a":1}"#),
        ("/api/settings/modules/1time", r#"{"a":1}"#),
        ("/api/settings/modules/has_underscore", r#"{"a":1}"#),
        (
            "/api/settings/modules/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            r#"{"a":1}"#,
        ),
        ("/api/settings/modules/time", "[1,2]"),
        ("/api/settings/modules/time", "\"text\""),
        ("/api/settings/modules/time", "null"),
    ];

    for (uri, body) in cases {
        let r = call(&app, "PUT", uri, Some(body)).await;
        assert_eq!(r.status, 400, "{uri} {body}");
        assert_eq!(r.content_type, "application/problem+json");
    }
    let r = call(
        &app,
        "PUT",
        "/api/settings/modules/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        Some("{}"),
    )
    .await;
    assert_eq!(r.status, 200, "32 chars is allowed");
    db.drop().await;
}
