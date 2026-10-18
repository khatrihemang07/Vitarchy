//! Fixtures: a stage database with the Time schema, a fake Toggl and a request helper.
#![allow(dead_code)]

pub mod activity;
pub mod clockify_snapshot;
pub mod snapshot;
pub mod timeline;
pub mod timeline_day;

use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use axum::{Router, body::Body, http::Request};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;
use vitarchy_core::{AppState, test_db::TestDb};
use vitarchy_time::toggl::{
    api::{TogglApi, TogglFactory},
    error::TogglError,
    types::{EntryQuery, Me, Raw, TogglClient, TogglEntry, TogglProject, TogglTag, Workspace},
};
use vitarchy_time::{SnapshotFile, Snapshots, Source};

pub const TOKEN: &str = "SECRETTOKEN123";

/// What the fake Toggl holds and has been asked.
pub struct World {
    pub valid_token: String,
    pub entries: Vec<Value>,
    pub projects: Vec<Value>,
    pub clients: Vec<Value>,
    pub tags: Vec<Value>,
    pub fail_with: Option<TogglError>,
    pub entry_queries: Vec<EntryQuery>,
    /// Who the token belongs to and which workspace is the default.
    pub user_id: i64,
    pub workspace_id: i64,
    /// When set, `time_entries` announces it was called and waits to be released.
    pub gate: Option<Gate>,
}

/// Holds a fake `time_entries` call open so a test can act while a Sync is mid-flight.
#[derive(Clone, Default)]
pub struct Gate {
    pub entered: Arc<tokio::sync::Notify>,
    pub release: Arc<tokio::sync::Notify>,
}

/// A fake Toggl, shared between the test and the app.
#[derive(Clone)]
pub struct FakeToggl(pub Arc<Mutex<World>>);

impl FakeToggl {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(World {
            valid_token: TOKEN.to_owned(),
            entries: vec![],
            projects: vec![json!({"id": 30, "name": "Deep work", "client_id": 20, "active": true})],
            clients: vec![json!({"id": 20, "name": "Acme", "archived": false})],
            tags: vec![json!({"id": 10, "name": "focus"})],
            fail_with: None,
            entry_queries: vec![],
            user_id: 5,
            workspace_id: 7,
            gate: None,
        })))
    }

    pub fn with<R>(&self, f: impl FnOnce(&mut World) -> R) -> R {
        f(&mut self.0.lock().unwrap())
    }

    pub fn factory(&self) -> TogglFactory {
        let world = self.0.clone();
        Arc::new(move |token| {
            Arc::new(FakeApi {
                world: world.clone(),
                token: token.to_owned(),
            })
        })
    }
}

struct FakeApi {
    world: Arc<Mutex<World>>,
    token: String,
}

impl FakeApi {
    fn check(&self) -> Result<std::sync::MutexGuard<'_, World>, TogglError> {
        let world = self.world.lock().unwrap();
        if world.valid_token != self.token {
            return Err(TogglError::InvalidToken);
        }
        match world.fail_with.clone() {
            Some(error) => Err(error),
            None => Ok(world),
        }
    }
}

#[async_trait]
impl TogglApi for FakeApi {
    async fn me(&self) -> Result<Me, TogglError> {
        let world = self.check()?;
        Ok(Me {
            id: world.user_id,
            default_workspace_id: world.workspace_id,
        })
    }
    async fn workspace(&self, _: i64) -> Result<Workspace, TogglError> {
        drop(self.check()?);
        Ok(Workspace {
            name: "My Workspace".to_owned(),
        })
    }
    async fn time_entries(&self, query: &EntryQuery) -> Result<Vec<Raw<TogglEntry>>, TogglError> {
        let (entries, gate) = {
            let mut world = self.check()?;
            world.entry_queries.push(query.clone());
            (world.entries.clone(), world.gate.clone())
        };
        if let Some(gate) = gate {
            gate.entered.notify_one();
            gate.release.notified().await;
        }
        Raw::parse_list(Value::Array(entries))
    }
    async fn projects(&self, _: i64) -> Result<Vec<Raw<TogglProject>>, TogglError> {
        Raw::parse_list(Value::Array(self.check()?.projects.clone()))
    }
    async fn clients(&self, _: i64) -> Result<Vec<Raw<TogglClient>>, TogglError> {
        Raw::parse_list(Value::Array(self.check()?.clients.clone()))
    }
    async fn tags(&self, _: i64) -> Result<Vec<Raw<TogglTag>>, TogglError> {
        Raw::parse_list(Value::Array(self.check()?.tags.clone()))
    }
}

/// A stopped one-hour entry. `description` is searched for in log assertions.
pub fn stopped_entry(id: i64, description: &str) -> Value {
    json!({
        "id": id, "workspace_id": 7, "project_id": 30, "description": description,
        "tags": ["focus"], "billable": false, "start": "2026-04-01T10:00:00+00:00",
        "stop": "2026-04-01T11:00:00+00:00", "duration": 3600,
        "at": "2026-04-01T11:00:01+00:00"
    })
}

/// A stopped one-hour entry that started `days_ago` days before now (inside a Sync's window).
pub fn recent_entry(id: i64, days_ago: i64) -> Value {
    let start = chrono::Utc::now() - chrono::Duration::days(days_ago);
    let mut entry = stopped_entry(id, "recent");
    entry["start"] = json!(start.to_rfc3339());
    entry["stop"] = json!((start + chrono::Duration::hours(1)).to_rfc3339());
    entry
}

/// The same entry still running.
pub fn running_entry(id: i64) -> Value {
    json!({
        "id": id, "workspace_id": 7, "description": "running", "tags": [],
        "start": "2026-04-02T10:00:00+00:00", "stop": null, "duration": -1_775_123_456
    })
}

/// A deletion marker, as a `since` query reports it.
pub fn deleted_entry(id: i64) -> Value {
    let mut entry = stopped_entry(id, "gone");
    entry["server_deleted_at"] = json!("2026-04-03T00:00:00+00:00");
    entry
}

/// A stage database with the Time migrations applied.
pub async fn time_db(label: &str) -> TestDb {
    let db = TestDb::create(&format!("time_{label}")).await;
    vitarchy_time::module()
        .migrator()
        .run(&db.pool)
        .await
        .unwrap();
    db
}

/// The Time routes over `db`, using `fake`, mounted as the server mounts them.
pub fn time_app(db: &TestDb, fake: &FakeToggl) -> Router {
    time_app_with_snapshots(db, fake, [])
}

/// Like [`time_app`], with each `(Source, Snapshot)` pointed at its fixture. Sources not
/// listed have no Snapshot.
pub fn time_app_with_snapshots(
    db: &TestDb,
    fake: &FakeToggl,
    files: impl IntoIterator<Item = (Source, SnapshotFile)>,
) -> Router {
    let snapshots = files.into_iter().fold(
        Snapshots::resolve(Some(Path::new("/nonexistent/vitarchy")), |_| None),
        |snapshots, (source, file)| snapshots.with(source, file),
    );
    let (router, _) = utoipa_axum::router::OpenApiRouter::new()
        .nest(
            "/api/time",
            vitarchy_time::router_with(fake.factory(), snapshots),
        )
        .split_for_parts();
    router.with_state(AppState::new(db.pool.clone()))
}

pub struct Reply {
    pub status: u16,
    pub content_type: String,
    pub text: String,
    pub json: Value,
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
        text: String::from_utf8_lossy(&bytes).into_owned(),
        json: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    }
}

/// Connects with the valid token.
pub async fn connect(app: &Router) {
    let body = json!({"api_token": TOKEN}).to_string();
    assert_eq!(
        call(app, "PUT", "/api/time/connection", Some(&body))
            .await
            .status,
        200
    );
}
