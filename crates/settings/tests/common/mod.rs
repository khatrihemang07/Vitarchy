//! Test fixtures: a throwaway database with the settings migrations applied,
//! and a request helper that talks to the real router.
#![allow(dead_code)]

use std::str::FromStr;

use axum::{Router, body::Body, http::Request};
use http_body_util::BodyExt;
use sqlx::{Connection, PgConnection, PgPool, postgres::PgConnectOptions};
use tower::ServiceExt;
use vitarchy_core::AppState;

fn database_url() -> String {
    dotenvy::dotenv().ok();
    std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (see .env.example)")
}

pub struct TempDb {
    name: String,
    pub pool: PgPool,
}

impl TempDb {
    /// Creates a database named after the test binary and migrates settings into it.
    pub async fn create(label: &str) -> Self {
        let name = format!("vitarchy_settings_{label}_{}", std::process::id());
        let opts = PgConnectOptions::from_str(&database_url()).unwrap();
        let mut admin = PgConnection::connect_with(&opts).await.unwrap();
        for sql in [
            format!("DROP DATABASE IF EXISTS {name} WITH (FORCE)"),
            format!("CREATE DATABASE {name}"),
        ] {
            sqlx::query(sqlx::AssertSqlSafe(sql))
                .execute(&mut admin)
                .await
                .unwrap();
        }
        let pool = PgPool::connect_with(opts.database(&name)).await.unwrap();
        vitarchy_settings::feature()
            .migrator()
            .run(&pool)
            .await
            .unwrap();
        Self { name, pool }
    }

    pub fn app(&self) -> Router {
        let (router, _) = utoipa_axum::router::OpenApiRouter::new()
            .nest("/api/settings", vitarchy_settings::feature().router())
            .split_for_parts();
        router.with_state(AppState::new(self.pool.clone()))
    }

    pub async fn drop(self) {
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

pub struct Reply {
    pub status: u16,
    pub content_type: String,
    pub json: serde_json::Value,
}

pub async fn call(app: &Router, method: &str, uri: &str, body: Option<&str>) -> Reply {
    let mut req = Request::builder().method(method).uri(uri);
    if body.is_some() {
        req = req.header("content-type", "application/json");
    }
    let res = app
        .clone()
        .oneshot(req.body(Body::from(body.unwrap_or("").to_owned())).unwrap())
        .await
        .unwrap();
    let status = res.status().as_u16();
    let content_type = res
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap().to_owned())
        .unwrap_or_default();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    Reply {
        status,
        content_type,
        json: serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null),
    }
}
