use crate::common::dummy_module;
use sqlx::PgPool;
use utoipa_axum::router::OpenApiRouter;
use vitarchy_core::{Module, test_db::TestDb};
use vitarchy_server::migrate;

fn second_module() -> Module {
    Module::new(
        "dummy2",
        OpenApiRouter::new,
        sqlx::migrate!("./tests/it/dummy2_migrations"),
    )
}

async fn table_exists(pool: &PgPool, schema: &str, table: &str) -> bool {
    sqlx::query_scalar("SELECT to_regclass($1) IS NOT NULL")
        .bind(format!("{schema}.{table}"))
        .fetch_one(pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn modules_share_one_tracking_table_and_each_creates_its_own_schema() {
    let db = TestDb::create("migrations").await;

    migrate::run(&db.pool, &[dummy_module(), second_module()])
        .await
        .unwrap();
    // Running again (a later boot) is a no-op, and one Module alone still works
    // even though the tracking table knows versions from the other.
    migrate::run(&db.pool, &[second_module()]).await.unwrap();
    migrate::run(&db.pool, &[dummy_module(), second_module()])
        .await
        .unwrap();

    assert!(table_exists(&db.pool, "dummy", "item").await);
    assert!(table_exists(&db.pool, "dummy2", "thing").await);
    let applied: i64 = sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(applied, 2);

    db.drop().await;
}
