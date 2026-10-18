//! The Timeline as an Import builds it: both Sources merged, repeatable, one Source followed.

use crate::common::{
    activity::{CLOCKIFY_IMPORT, clockify_app, stored_for},
    call,
    clockify_snapshot::{ClockifyFixture, Item},
    snapshot::Fixture,
    time_db,
    timeline::{both_app, has_overlap, import_both, segments, write_both},
};

#[tokio::test]
async fn importing_both_sources_builds_a_timeline_that_never_overlaps_and_knows_when_away() {
    let db = time_db("tlboth").await;
    let (toggl, clockify) = (Fixture::new(), ClockifyFixture::new());
    write_both(&toggl, &clockify);

    import_both(&both_app(&db, toggl.file(), clockify.file())).await;

    let stored = segments(&db).await;
    let shape: Vec<_> = stored
        .iter()
        .map(|s| (s.start, s.end, s.away.as_str(), s.sources.len()))
        .collect();
    assert_eq!(
        shape,
        [
            (0, 50, "unknown", 1),
            (50, 100, "no", 2),
            (100, 3650, "no", 1),
            (3650, 7250, "yes", 1)
        ]
    );
    assert!(!has_overlap(&db).await);
    // Toggl's title and Clockify's url both survive where the Sources overlap.
    assert_eq!(stored[1].title, "T");
    assert_eq!(stored[1].url.as_deref(), Some("https://example.com"));
    assert_eq!(stored[0].url, None);
    db.drop().await;
}

#[tokio::test]
async fn importing_again_leaves_the_same_segments() {
    let db = time_db("tlagain").await;
    let (toggl, clockify) = (Fixture::new(), ClockifyFixture::new());
    write_both(&toggl, &clockify);
    let app = both_app(&db, toggl.file(), clockify.file());
    import_both(&app).await;
    let before = segments(&db).await;

    import_both(&app).await;

    assert_eq!(segments(&db).await, before);
    db.drop().await;
}

#[tokio::test]
async fn with_one_source_the_timeline_follows_it() {
    let db = time_db("tlone").await;
    let clockify = ClockifyFixture::new();
    let mut item = Item::new("c1", 0, 7200);
    item.idle = 3600.0;
    clockify.write(&[item]);
    let app = clockify_app(&db, clockify.file());

    call(&app, "POST", CLOCKIFY_IMPORT, None).await;

    let shape: Vec<_> = segments(&db)
        .await
        .into_iter()
        .map(|s| (s.start, s.end, s.away, s.sources))
        .collect();
    assert_eq!(
        shape,
        [
            (0, 3600, "no".to_owned(), vec!["clockify".to_owned()]),
            (3600, 7200, "yes".to_owned(), vec!["clockify".to_owned()]),
        ]
    );
    db.drop().await;
}

/// Imports one Clockify row of `length` seconds with `idle` of no input; returns its Segments.
async fn import_idle_row(name: &str, length: i64, idle: f64) -> Vec<(i64, i64, String)> {
    let db = time_db(name).await;
    let clockify = ClockifyFixture::new();
    let mut item = Item::new("c1", 0, length);
    item.idle = idle;
    clockify.write(&[item]);
    call(
        &clockify_app(&db, clockify.file()),
        "POST",
        CLOCKIFY_IMPORT,
        None,
    )
    .await;
    let shape = segments(&db)
        .await
        .into_iter()
        .map(|s| (s.start, s.end, s.away))
        .collect();
    db.drop().await;
    shape
}

#[tokio::test]
async fn thirty_minutes_without_input_is_not_away() {
    let shape = import_idle_row("tl30", 1800, 1800.0).await;
    assert_eq!(shape, [(0, 1800, "no".to_owned())]);
}

#[tokio::test]
async fn ninety_minutes_without_input_is_all_away() {
    let shape = import_idle_row("tl90", 5400, 5400.0).await;
    assert_eq!(shape, [(0, 5400, "yes".to_owned())]);
}

#[tokio::test]
async fn an_import_records_its_run_and_builds_the_timeline_together() {
    let db = time_db("tlrun").await;
    let clockify = ClockifyFixture::new();
    clockify.write(&[Item::new("a", 0, 60)]);
    let app = clockify_app(&db, clockify.file());

    let reply = call(&app, "POST", CLOCKIFY_IMPORT, None).await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    let runs: i64 = sqlx::query_scalar("SELECT count(*) FROM time.runs WHERE kind = 'import'")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!((runs, segments(&db).await.len()), (1, 1));
    db.drop().await;
}

#[tokio::test]
async fn a_failed_timeline_build_undoes_the_import_so_the_next_one_retries_both() {
    let db = time_db("tlatomic").await;
    let clockify = ClockifyFixture::new();
    clockify.write(&[Item::new("a", 0, 60)]);
    let app = clockify_app(&db, clockify.file());
    for statement in [
        "CREATE FUNCTION time.refuse_segments() RETURNS trigger LANGUAGE plpgsql
           AS $$ BEGIN RAISE EXCEPTION 'segments refused'; END $$",
        "CREATE TRIGGER refuse BEFORE INSERT ON time.timeline_segments
           FOR EACH ROW EXECUTE FUNCTION time.refuse_segments()",
    ] {
        sqlx::query(statement).execute(&db.pool).await.unwrap();
    }

    let failed = call(&app, "POST", CLOCKIFY_IMPORT, None).await;

    assert_eq!(failed.status, 500, "{}", failed.text);
    let stored = stored_for(&db, "clockify").await;
    assert_eq!((stored, segments(&db).await.len()), (0, 0));
    sqlx::query("DROP TRIGGER refuse ON time.timeline_segments")
        .execute(&db.pool)
        .await
        .unwrap();

    let retried = call(&app, "POST", CLOCKIFY_IMPORT, None).await;

    assert_eq!(retried.status, 200, "{}", retried.text);
    assert_eq!(retried.json["inserted"], 1);
    assert_eq!(segments(&db).await.len(), 1);
    db.drop().await;
}
