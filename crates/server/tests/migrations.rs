mod common;

use common::{database_url, dummy_module};
use sqlx::{Connection, PgConnection, PgPool, postgres::PgConnectOptions};
use std::str::FromStr;
use utoipa_axum::router::OpenApiRouter;
use vitarchy_core::Module;
use vitarchy_server::migrate;

fn second_module() -> Module {
    Module::new(
        "dummy2",
        OpenApiRouter::new,
        sqlx::migrate!("./tests/dummy2_migrations"),
    )
}

/// A throwaway database so the test never touches real data.
struct TempDb {
    name: String,
    pool: PgPool,
}

impl TempDb {
    async fn create() -> Self {
        let name = format!("vitarchy_test_{}", std::process::id());
        let opts = PgConnectOptions::from_str(&database_url()).unwrap();
        let mut admin = PgConnection::connect_with(&opts).await.unwrap();
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "DROP DATABASE IF EXISTS {name} WITH (FORCE)"
        )))
        .execute(&mut admin)
        .await
        .unwrap();
        sqlx::query(sqlx::AssertSqlSafe(format!("CREATE DATABASE {name}")))
            .execute(&mut admin)
            .await
            .unwrap();
        let pool = PgPool::connect_with(opts.database(&name)).await.unwrap();
        Self { name, pool }
    }

    async fn drop(self) {
        self.pool.close().await;
        let opts = PgConnectOptions::from_str(&database_url()).unwrap();
        let mut admin = PgConnection::connect_with(&opts).await.unwrap();
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "DROP DATABASE {} WITH (FORCE)",
            self.name
        )))
        .execute(&mut admin)
        .await
        .unwrap();
    }
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
    let db = TempDb::create().await;

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
