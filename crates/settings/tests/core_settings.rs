mod common;

use common::{call, settings_app, settings_db};
use serde_json::json;

fn defaults() -> serde_json::Value {
    json!({
        "display_name": "",
        "theme": "system",
        "motion": "system",
        "week_start": "monday",
        "clock": "system",
        "date_format": "system"
    })
}

#[tokio::test]
async fn empty_database_returns_defaults() {
    let db = settings_db("defaults").await;

    let r = call(&settings_app(&db), "GET", "/api/settings", None).await;

    assert_eq!(r.status, 200);
    assert_eq!(r.json, json!({"core": defaults(), "modules": {}}));
    db.drop().await;
}

#[tokio::test]
async fn saved_core_settings_read_back_and_put_returns_them() {
    let db = settings_db("roundtrip").await;
    let app = settings_app(&db);
    let body = json!({
        "display_name": "Hemang", "theme": "dark", "motion": "reduced",
        "week_start": "sunday", "clock": "h24", "date_format": "day_month_year"
    });

    let put = call(&app, "PUT", "/api/settings/core", Some(&body.to_string())).await;
    let get = call(&app, "GET", "/api/settings", None).await;

    assert_eq!(put.status, 200);
    assert_eq!(put.json, body);
    assert_eq!(get.json["core"], body);
    db.drop().await;
}

#[tokio::test]
async fn partial_body_and_partial_stored_object_fill_in_defaults() {
    let db = settings_db("partial").await;
    let app = settings_app(&db);

    let put = call(
        &app,
        "PUT",
        "/api/settings/core",
        Some(r#"{"theme":"light"}"#),
    )
    .await;
    assert_eq!(put.json["theme"], "light");
    assert_eq!(put.json["clock"], "system");

    sqlx::query("UPDATE settings.entries SET value = '{\"clock\":\"h12\"}' WHERE scope = 'core'")
        .execute(&db.pool)
        .await
        .unwrap();
    let get = call(&app, "GET", "/api/settings", None).await;
    assert_eq!(get.json["core"]["clock"], "h12");
    assert_eq!(get.json["core"]["theme"], "system");
    db.drop().await;
}

#[tokio::test]
async fn display_name_is_trimmed() {
    let db = settings_db("trim").await;

    let r = call(
        &settings_app(&db),
        "PUT",
        "/api/settings/core",
        Some(r#"{"display_name":"  Ada  "}"#),
    )
    .await;

    assert_eq!(r.json["display_name"], "Ada");
    db.drop().await;
}

#[tokio::test]
async fn invalid_core_input_is_a_400_problem_and_nothing_is_saved() {
    let db = settings_db("invalid").await;
    let app = settings_app(&db);
    let too_long = json!({"display_name": "x".repeat(51)}).to_string();
    let bad = [r#"{"theme":"purple"}"#, r"{not json", too_long.as_str()];

    for body in bad {
        let r = call(&app, "PUT", "/api/settings/core", Some(body)).await;
        assert_eq!(r.status, 400, "{body}");
        assert_eq!(r.content_type, "application/problem+json");
        assert_eq!(r.json["title"], "Bad Request");
    }
    let get = call(&app, "GET", "/api/settings", None).await;
    assert_eq!(get.json["core"], defaults());
    let ok = json!({"display_name": "x".repeat(50)}).to_string();
    assert_eq!(
        call(&app, "PUT", "/api/settings/core", Some(&ok))
            .await
            .status,
        200
    );
    db.drop().await;
}

#[tokio::test]
async fn one_bad_stored_field_resets_only_that_field() {
    let db = settings_db("lenient").await;
    let stored = r#"{"theme":"purple","clock":"h24","display_name":"Ada","motion":7}"#;
    sqlx::query("INSERT INTO settings.entries (scope, value) VALUES ('core', $1::jsonb)")
        .bind(stored)
        .execute(&db.pool)
        .await
        .unwrap();

    let get = call(&settings_app(&db), "GET", "/api/settings", None).await;

    let core = &get.json["core"];
    assert_eq!(core["theme"], "system");
    assert_eq!(core["motion"], "system");
    assert_eq!(core["clock"], "h24");
    assert_eq!(core["display_name"], "Ada");
    db.drop().await;
}
