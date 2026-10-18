//! The Timeline's figures in the status response.

use crate::common::{
    activity::{CLOCKIFY_IMPORT, clockify_app},
    call,
    clockify_snapshot::{ClockifyFixture, Item},
    snapshot::Fixture,
    time_db,
    timeline::{both_app, import_both, write_both},
};

#[tokio::test]
async fn the_status_reports_the_timeline() {
    let db = time_db("tlstatus").await;
    let (toggl, clockify) = (Fixture::new(), ClockifyFixture::new());
    write_both(&toggl, &clockify);
    let app = both_app(&db, toggl.file(), clockify.file());
    let empty = call(&app, "GET", "/api/time/status", None).await;
    assert_eq!(empty.json["timeline"]["segment_count"], 0);
    assert!(empty.json["timeline"]["oldest"].is_null());
    import_both(&app).await;

    let reply = call(&app, "GET", "/api/time/status", None).await;

    let timeline = &reply.json["timeline"];
    assert_eq!(timeline["segment_count"], 4);
    assert_eq!(
        (
            timeline["away_secs"].as_i64(),
            timeline["unknown_secs"].as_i64()
        ),
        (Some(3600), Some(50))
    );
    assert_eq!(timeline["covered_secs"], 7250);
    assert!(timeline["oldest"].is_string() && timeline["newest"].is_string());
    db.drop().await;
}

#[tokio::test]
async fn covered_time_leaves_out_the_gaps_between_segments() {
    let db = time_db("tlcovered").await;
    let clockify = ClockifyFixture::new();
    clockify.write(&[Item::new("a", 0, 60), Item::new("b", 1000, 1100)]);
    let app = clockify_app(&db, clockify.file());
    call(&app, "POST", CLOCKIFY_IMPORT, None).await;

    let reply = call(&app, "GET", "/api/time/status", None).await;

    assert_eq!(reply.json["timeline"]["covered_secs"], 160);
    db.drop().await;
}
