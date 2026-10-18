//! The Timeline kept current as Activity changes, and rebuilt on request.

use crate::common::{
    activity::{CLOCKIFY_IMPORT, clockify_app},
    call,
    clockify_snapshot::{ClockifyFixture, Item},
    snapshot::Fixture,
    time_db,
    timeline::{both_app, import_both, segments, write_both},
};

async fn rebuild_everything(app: &axum::Router) {
    let reply = call(app, "POST", "/api/time/timeline/rebuild", None).await;
    assert_eq!(reply.status, 200, "{}", reply.text);
}

#[tokio::test]
async fn a_grown_activity_rebuilds_its_stretch_only() {
    let db = time_db("tlgrow").await;
    let clockify = ClockifyFixture::new();
    clockify.write(&[Item::new("a", 0, 60), Item::new("b", 1000, 1100)]);
    let app = clockify_app(&db, clockify.file());
    call(&app, "POST", CLOCKIFY_IMPORT, None).await;

    clockify.write(&[Item::new("a", 0, 90), Item::new("b", 1000, 1100)]);
    call(&app, "POST", CLOCKIFY_IMPORT, None).await;

    let ends: Vec<_> = segments(&db)
        .await
        .iter()
        .map(|s| (s.start, s.end))
        .collect();
    assert_eq!(ends, [(0, 90), (1000, 1100)]);
    db.drop().await;
}

#[tokio::test]
async fn the_rebuild_route_restores_a_lost_timeline_and_reports_its_size() {
    let db = time_db("tlroute").await;
    let (toggl, clockify) = (Fixture::new(), ClockifyFixture::new());
    write_both(&toggl, &clockify);
    let app = both_app(&db, toggl.file(), clockify.file());
    import_both(&app).await;
    let before = segments(&db).await;
    sqlx::query("DELETE FROM time.timeline_segments")
        .execute(&db.pool)
        .await
        .unwrap();

    let reply = call(&app, "POST", "/api/time/timeline/rebuild", None).await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    assert_eq!(reply.json["segment_count"], 4);
    assert_eq!(reply.json["activity_count"], 2);
    assert_eq!(segments(&db).await, before);
    db.drop().await;
}

#[tokio::test]
async fn rebuilding_with_no_activity_gives_an_empty_timeline() {
    let db = time_db("tlempty").await;
    let app = both_app(&db, Fixture::new().file(), ClockifyFixture::new().file());

    let reply = call(&app, "POST", "/api/time/timeline/rebuild", None).await;

    assert_eq!(
        (reply.status, reply.json["segment_count"].as_u64()),
        (200, Some(0))
    );
    db.drop().await;
}

#[tokio::test]
async fn a_grown_activity_keeps_the_touching_activity_before_it_in_the_timeline() {
    let db = time_db("tltouch").await;
    let clockify = ClockifyFixture::new();
    clockify.write(&[Item::new("a", 0, 600), Item::new("b", 600, 1200)]);
    let app = clockify_app(&db, clockify.file());
    call(&app, "POST", CLOCKIFY_IMPORT, None).await;
    assert_eq!(segments(&db).await.len(), 1);

    clockify.write(&[Item::new("a", 0, 600), Item::new("b", 600, 1800)]);
    call(&app, "POST", CLOCKIFY_IMPORT, None).await;

    let grown = segments(&db).await;
    assert_eq!(
        grown.iter().map(|s| (s.start, s.end)).collect::<Vec<_>>(),
        [(0, 1800)]
    );
    rebuild_everything(&app).await;
    assert_eq!(segments(&db).await, grown);
    db.drop().await;
}

#[tokio::test]
async fn a_new_activity_joins_the_equal_segment_it_touches() {
    let db = time_db("tljoin").await;
    let clockify = ClockifyFixture::new();
    clockify.write(&[Item::new("a", 0, 600), Item::new("c", 2000, 2100)]);
    let app = clockify_app(&db, clockify.file());
    call(&app, "POST", CLOCKIFY_IMPORT, None).await;

    clockify.write(&[
        Item::new("a", 0, 600),
        Item::new("b", 600, 900),
        Item::new("c", 2000, 2100),
    ]);
    call(&app, "POST", CLOCKIFY_IMPORT, None).await;

    let ends: Vec<_> = segments(&db)
        .await
        .iter()
        .map(|s| (s.start, s.end))
        .collect();
    assert_eq!(ends, [(0, 900), (2000, 2100)]);
    db.drop().await;
}

#[tokio::test]
async fn rebuilding_the_middle_leaves_segments_outside_it_untouched() {
    let db = time_db("tlmiddle").await;
    let clockify = ClockifyFixture::new();
    let mut other = Item::new("b", 600, 900);
    other.name = "Mail";
    clockify.write(&[Item::new("a", 0, 600), other, Item::new("c", 2000, 2100)]);
    let app = clockify_app(&db, clockify.file());
    call(&app, "POST", CLOCKIFY_IMPORT, None).await;
    let before = segments(&db).await;
    assert_eq!(before.len(), 3);
    sqlx::query("UPDATE time.timeline_segments SET title = 'marker' WHERE start_at <> $1")
        .bind(chrono::DateTime::from_timestamp(978_307_200 + 600, 0).unwrap())
        .execute(&db.pool)
        .await
        .unwrap();

    let mut grown = Item::new("b", 600, 1000);
    grown.name = "Mail";
    clockify.write(&[Item::new("a", 0, 600), grown, Item::new("c", 2000, 2100)]);
    call(&app, "POST", CLOCKIFY_IMPORT, None).await;

    let after = segments(&db).await;
    assert_eq!(after.len(), 3);
    assert_eq!((after[1].start, after[1].end), (600, 1000));
    assert_eq!(
        (&after[0].title[..], &after[2].title[..]),
        ("marker", "marker")
    );
    db.drop().await;
}

#[tokio::test]
async fn a_shrunk_activity_leaves_no_segment_where_it_no_longer_reaches() {
    let db = time_db("tlshrink").await;
    let clockify = ClockifyFixture::new();
    clockify.write(&[Item::new("a", 0, 600), Item::new("b", 2000, 2100)]);
    let app = clockify_app(&db, clockify.file());
    call(&app, "POST", CLOCKIFY_IMPORT, None).await;

    clockify.write(&[Item::new("a", 0, 100), Item::new("b", 2000, 2100)]);
    call(&app, "POST", CLOCKIFY_IMPORT, None).await;

    let imported = segments(&db).await;
    assert_eq!(
        imported
            .iter()
            .map(|s| (s.start, s.end))
            .collect::<Vec<_>>(),
        [(0, 100), (2000, 2100)]
    );
    rebuild_everything(&app).await;
    assert_eq!(segments(&db).await, imported);
    db.drop().await;
}

#[tokio::test]
async fn a_moved_activity_clears_where_it_was_and_fills_where_it_is() {
    let db = time_db("tlmove").await;
    let clockify = ClockifyFixture::new();
    clockify.write(&[Item::new("a", 0, 100)]);
    let app = clockify_app(&db, clockify.file());
    call(&app, "POST", CLOCKIFY_IMPORT, None).await;

    clockify.write(&[Item::new("a", 5000, 5100)]);
    call(&app, "POST", CLOCKIFY_IMPORT, None).await;

    let imported = segments(&db).await;
    assert_eq!(
        imported
            .iter()
            .map(|s| (s.start, s.end))
            .collect::<Vec<_>>(),
        [(5000, 5100)]
    );
    rebuild_everything(&app).await;
    assert_eq!(segments(&db).await, imported);
    db.drop().await;
}

fn item(id: &str, started: i64, ended: i64, idle: f64) -> Item {
    let mut item = Item::new(id, started, ended);
    item.idle = idle;
    item
}

/// Imports `first`, then `second` (only the later row differs), and checks the partial rebuild
/// the second Import did equals rebuilding everything.
async fn partial_rebuild_equals_rebuild_all(name: &str, first: &[Item], second: &[Item]) {
    let db = time_db(name).await;
    let clockify = ClockifyFixture::new();
    clockify.write(first);
    let app = clockify_app(&db, clockify.file());
    call(&app, "POST", CLOCKIFY_IMPORT, None).await;

    clockify.write(second);
    call(&app, "POST", CLOCKIFY_IMPORT, None).await;
    let partial = segments(&db).await;
    rebuild_everything(&app).await;

    assert_eq!(partial, segments(&db).await);
    let shape: Vec<_> = partial
        .iter()
        .map(|s| (s.start, s.end, s.away.as_str()))
        .collect();
    assert_eq!(shape, [(0, 3900, "yes")]);
    db.drop().await;
}

#[tokio::test]
async fn a_grown_row_makes_the_rows_before_it_in_its_no_input_stretch_away_like_a_full_rebuild() {
    partial_rebuild_equals_rebuild_all(
        "tlstretch_grow",
        &[
            item("a", 0, 1200, 1200.0),
            item("b", 1200, 2400, 2400.0),
            item("c", 2400, 3000, 3000.0),
        ],
        &[
            item("a", 0, 1200, 1200.0),
            item("b", 1200, 2400, 2400.0),
            item("c", 2400, 3900, 3900.0),
        ],
    )
    .await;
}

#[tokio::test]
async fn a_new_later_row_makes_the_rows_before_it_in_its_no_input_stretch_away_like_a_full_rebuild()
{
    partial_rebuild_equals_rebuild_all(
        "tlstretch_new",
        &[item("a", 0, 1200, 1200.0), item("b", 1200, 2400, 2400.0)],
        &[
            item("a", 0, 1200, 1200.0),
            item("b", 1200, 2400, 2400.0),
            item("c", 2400, 3900, 3900.0),
        ],
    )
    .await;
}
