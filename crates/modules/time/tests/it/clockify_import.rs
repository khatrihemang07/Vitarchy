//! The Clockify Activity Import route against a fixture Clockify Snapshot.

use crate::common::{
    activity::{CLOCKIFY_IMPORT, clockify_app, stored_for},
    call,
    clockify_snapshot::{ClockifyFixture, Item},
    time_db,
};

#[tokio::test]
async fn a_clockify_import_stores_url_idle_seconds_and_raw_values_but_no_icon_or_toggl_extras() {
    let db = time_db("clkinsert").await;
    let snap = ClockifyFixture::new();
    let mut browser = Item::new("b", 100, 160);
    browser.name = "Brave Browser";
    browser.url = "https://example.com";
    browser.idle = 33.0;
    snap.write(&[Item::new("a", 0, 60), browser]);
    let app = clockify_app(&db, snap.file());

    let reply = call(&app, "POST", CLOCKIFY_IMPORT, None).await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    assert_eq!(
        (reply.json["read"].as_u64(), reply.json["inserted"].as_u64()),
        (Some(2), Some(2))
    );
    #[allow(clippy::type_complexity)]
    let (url, idle_secs, idle, client, end, has_icon): (
        Option<String>,
        Option<f64>,
        Option<bool>,
        Option<String>,
        String,
        bool,
    ) = sqlx::query_as(
        "SELECT url, idle_secs, idle, client_id::text, end_at::text, raw ? 'ZICONDATA'
         FROM time.activities WHERE source = 'clockify' AND source_id = 'b'",
    )
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(url.as_deref(), Some("https://example.com"));
    assert_eq!(idle_secs, Some(33.0));
    assert_eq!((idle, client), (None, None));
    assert_eq!(end, "2001-01-01 00:02:40+00");
    assert!(!has_icon);
    db.drop().await;
}

#[tokio::test]
async fn clockify_importing_again_changes_nothing_and_a_grown_item_gets_its_new_end() {
    let db = time_db("clkagain").await;
    let snap = ClockifyFixture::new();
    snap.write(&[Item::new("a", 0, 60), Item::new("b", 70, 80)]);
    let app = clockify_app(&db, snap.file());
    call(&app, "POST", CLOCKIFY_IMPORT, None).await;

    let again = call(&app, "POST", CLOCKIFY_IMPORT, None).await;
    snap.write(&[Item::new("a", 0, 60), Item::new("b", 70, 600)]);
    let grown = call(&app, "POST", CLOCKIFY_IMPORT, None).await;

    assert_eq!(
        (
            again.json["inserted"].as_u64(),
            again.json["unchanged"].as_u64()
        ),
        (Some(0), Some(2))
    );
    assert_eq!(
        (
            grown.json["updated"].as_u64(),
            grown.json["unchanged"].as_u64()
        ),
        (Some(1), Some(1))
    );
    assert_eq!(stored_for(&db, "clockify").await, 2);
    db.drop().await;
}

#[tokio::test]
async fn clockify_items_missing_from_a_newer_snapshot_are_kept() {
    let db = time_db("clkkeep").await;
    let snap = ClockifyFixture::new();
    snap.write(&[Item::new("a", 0, 60)]);
    let app = clockify_app(&db, snap.file());
    call(&app, "POST", CLOCKIFY_IMPORT, None).await;
    snap.write(&[Item::new("b", 70, 80)]);

    call(&app, "POST", CLOCKIFY_IMPORT, None).await;

    assert_eq!(stored_for(&db, "clockify").await, 2);
    db.drop().await;
}

#[tokio::test]
async fn unusable_clockify_items_are_skipped_and_counted() {
    let db = time_db("clkskip").await;
    let snap = ClockifyFixture::new();
    snap.write(&[Item::new("a", 0, 60)]);
    snap.add_backwards_item();

    let reply = call(
        &clockify_app(&db, snap.file()),
        "POST",
        CLOCKIFY_IMPORT,
        None,
    )
    .await;

    assert_eq!(
        (reply.json["read"].as_u64(), reply.json["skipped"].as_u64()),
        (Some(2), Some(1))
    );
    db.drop().await;
}

// ---- Several Sources ----
