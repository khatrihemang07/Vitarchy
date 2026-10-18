//! The Activity Sources migration on a database that already holds Toggl data.

use sqlx::migrate::Migrator;
use vitarchy_core::test_db::TestDb;

const SOURCES_MIGRATION: i64 = 20_260_404_000_000;

/// Applies only the migrations before the Activity Sources one, as the live database had them.
async fn migrate_before_sources(db: &TestDb) {
    let earlier = vitarchy_time::module()
        .migrator()
        .iter()
        .filter(|migration| migration.version < SOURCES_MIGRATION)
        .cloned()
        .collect::<Vec<_>>();
    let migrator = Migrator {
        migrations: std::borrow::Cow::Owned(earlier),
        ..Migrator::DEFAULT
    };
    migrator.run(&db.pool).await.unwrap();
}

#[tokio::test]
async fn existing_toggl_rows_and_import_runs_become_the_toggl_source() {
    let db = TestDb::create("time_migrate_src").await;
    migrate_before_sources(&db).await;
    sqlx::query(
        "INSERT INTO time.activities (toggl_id, client_id, app, title, start_at, end_at, idle, raw)
         VALUES ('12345678-9ABC-DEF0-0102-030405060701', '12345678-9abc-def0-0102-030405060700',
                 'App', 't', '2026-01-01T00:00:00Z', '2026-01-01T00:01:00Z', true, '{}')",
    )
    .execute(&db.pool)
    .await
    .unwrap();
    for kind in ["sync", "import"] {
        sqlx::query(
            "INSERT INTO time.runs (kind, started_at, finished_at, outcome)
             VALUES ($1, now(), now(), 'success')",
        )
        .bind(kind)
        .execute(&db.pool)
        .await
        .unwrap();
    }

    vitarchy_time::module()
        .migrator()
        .run(&db.pool)
        .await
        .unwrap();

    let row: (String, String, Option<bool>, Option<String>, Option<f64>) =
        sqlx::query_as("SELECT source, source_id, idle, url, idle_secs FROM time.activities")
            .fetch_one(&db.pool)
            .await
            .unwrap();
    assert_eq!(
        row,
        (
            "toggl".into(),
            "12345678-9abc-def0-0102-030405060701".into(),
            Some(true),
            None,
            None
        )
    );
    let runs: Vec<(String, Option<String>)> =
        sqlx::query_as("SELECT kind, source FROM time.runs ORDER BY kind")
            .fetch_all(&db.pool)
            .await
            .unwrap();
    assert_eq!(
        runs,
        [
            ("import".into(), Some("toggl".into())),
            ("sync".into(), None)
        ]
    );
    db.drop().await;
}

#[tokio::test]
async fn a_row_with_the_other_sources_columns_is_refused() {
    let db = TestDb::create("time_migrate_chk").await;
    vitarchy_time::module()
        .migrator()
        .run(&db.pool)
        .await
        .unwrap();
    let toggl_with_url = sqlx::query(
        "INSERT INTO time.activities (source, source_id, app, title, start_at, end_at, idle, url, raw)
         VALUES ('toggl', 'a', 'App', '', now(), now(), false, 'https://x', '{}')",
    )
    .execute(&db.pool)
    .await;
    let clockify_with_idle = sqlx::query(
        "INSERT INTO time.activities (source, source_id, app, title, start_at, end_at, idle, idle_secs, raw)
         VALUES ('clockify', 'a', 'App', '', now(), now(), false, 0, '{}')",
    )
    .execute(&db.pool)
    .await;
    let run_without_source = sqlx::query(
        "INSERT INTO time.runs (kind, started_at, finished_at, outcome)
         VALUES ('import', now(), now(), 'success')",
    )
    .execute(&db.pool)
    .await;
    assert!(toggl_with_url.is_err());
    assert!(clockify_with_idle.is_err());
    assert!(run_without_source.is_err());
    db.drop().await;
}
