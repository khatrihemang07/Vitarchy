//! `GET /api/time/timeline/away-preview`: what each Away threshold would give.

mod common;

use common::{
    activity::{CLOCKIFY_IMPORT, clockify_app},
    call,
    clockify_snapshot::{ClockifyFixture, Item},
    time_db,
};

const PREVIEW: &str = "/api/time/timeline/away-preview";

#[tokio::test]
async fn the_preview_returns_every_choice_with_the_current_rules() {
    let db = time_db("awprev").await;
    let clockify = ClockifyFixture::new();
    clockify.write(&[]);
    let app = clockify_app(&db, clockify.file());

    let reply = call(&app, "GET", PREVIEW, None).await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    assert_eq!(reply.json["current_secs"], 3600);
    assert_eq!(reply.json["watching_cap_secs"], 7200);
    assert_eq!(reply.json["choices"].as_array().unwrap().len(), 8);
    assert_eq!(reply.json["days"], 28);
    assert!(
        reply.json["watching_apps"]
            .as_array()
            .unwrap()
            .contains(&"VLC".into())
    );
    assert!(
        reply.json["watching_sites"]
            .as_array()
            .unwrap()
            .contains(&"youtube.com".into())
    );
    assert!(!reply.text.to_lowercase().contains("spotify"));
    db.drop().await;
}

#[tokio::test]
async fn recent_clockify_activity_counts_and_old_activity_does_not() {
    let db = time_db("awprev_window").await;
    let clockify = ClockifyFixture::new();
    let mut recent = Item::new("recent", 0, 7200);
    recent.idle = 3600.0;
    clockify.write(&[recent]);
    let app = clockify_app(&db, clockify.file());
    call(&app, "POST", CLOCKIFY_IMPORT, None).await;
    // Fixture times are Core Data seconds from 2001, so the row is far older than 28 Days.
    let old = call(&app, "GET", PREVIEW, None).await;
    assert_eq!(old.json["choices"][7]["away_secs_per_day"], 0);

    sqlx::query("UPDATE time.activities SET start_at = now() - interval '3 hours', end_at = now() - interval '1 hour'")
        .execute(&db.pool)
        .await
        .unwrap();
    let recent = call(&app, "GET", PREVIEW, None).await;

    assert_eq!(recent.json["choices"][7]["away_secs_per_day"], 3600);
    assert_eq!(recent.json["choices"][7]["active_secs_per_day"], 3600);
    db.drop().await;
}
