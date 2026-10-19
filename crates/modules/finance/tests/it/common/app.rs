//! The Finance routes over a stage database, with a fake reader and a throwaway file folder.

use std::{path::PathBuf, sync::Arc};

use axum::{Router, body::Body, http::Request};
use http_body_util::BodyExt;
use serde_json::Value;
use tempfile::TempDir;
use tower::ServiceExt;
use vitarchy_core::{AppState, test_db::TestDb};
use vitarchy_finance::{
    files::FileStore,
    reader::{ReaderResult, testing::FakeReader},
};

use super::finance_db;

pub struct Reply {
    pub status: u16,
    pub text: String,
    pub json: Value,
}

/// One test's world: database, fake reader, file folder and the routes mounted as the server
/// mounts them.
pub struct Env {
    pub db: TestDb,
    pub reader: Arc<FakeReader>,
    pub dir: TempDir,
    pub app: Router,
}

impl Env {
    /// The reader answers `results` in order, then repeats the last.
    pub async fn new(label: &str, results: Vec<ReaderResult>) -> Self {
        Self::with_reader(label, FakeReader::with_results(results)).await
    }

    pub async fn with_reader(label: &str, reader: FakeReader) -> Self {
        Self::on_database(finance_db(label).await, reader)
    }

    /// The routes over an existing, already migrated database.
    pub fn on_database(db: TestDb, reader: FakeReader) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let reader = Arc::new(reader);
        let (router, _) = utoipa_axum::router::OpenApiRouter::new()
            .nest(
                "/api/finance",
                vitarchy_finance::router_with(reader.clone(), FileStore::new(dir.path())),
            )
            .split_for_parts();
        let app = router.with_state(AppState::new(db.pool.clone()));
        Self {
            db,
            reader,
            dir,
            app,
        }
    }

    pub fn store(&self) -> FileStore {
        FileStore::new(self.dir.path())
    }

    pub fn stored_file(&self, sha: &str) -> PathBuf {
        self.store().path(sha)
    }

    pub async fn get(&self, uri: &str) -> Reply {
        self.json_call("GET", uri, None).await
    }

    pub async fn post(&self, uri: &str, body: &str) -> Reply {
        self.json_call("POST", uri, Some(body)).await
    }

    /// A POST carrying an `x-request-id`, as the server's request-id layer would add.
    pub async fn post_with_request_id(&self, uri: &str, request_id: &str) -> Reply {
        self.call("POST", uri, Some("{}"), Some(request_id)).await
    }

    /// A call with a body carrying an `x-request-id`, as the server's request-id layer would add.
    pub async fn call_with_request_id(
        &self,
        method: &str,
        uri: &str,
        body: &str,
        request_id: &str,
    ) -> Reply {
        self.call(method, uri, Some(body), Some(request_id)).await
    }

    pub async fn json_call(&self, method: &str, uri: &str, body: Option<&str>) -> Reply {
        self.call(method, uri, body, None).await
    }

    async fn call(
        &self,
        method: &str,
        uri: &str,
        body: Option<&str>,
        request_id: Option<&str>,
    ) -> Reply {
        let mut req = Request::builder()
            .method(method)
            .uri(format!("/api/finance{uri}"));
        if let Some(id) = request_id {
            req = req.header("x-request-id", id);
        }
        if body.is_some() {
            req = req.header("content-type", "application/json");
        }
        let body = Body::from(body.unwrap_or("").to_owned());
        reply(
            self.app
                .clone()
                .oneshot(req.body(body).unwrap())
                .await
                .unwrap(),
        )
        .await
    }

    /// Uploads files as `multipart/form-data`.
    pub async fn upload(&self, files: &[(&str, &[u8])]) -> Reply {
        let boundary = "TESTBOUNDARY7MA4YWxk";
        let mut body = Vec::new();
        for (name, bytes) in files {
            body.extend_from_slice(
                format!(
                    "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{name}\"\r\nContent-Type: application/octet-stream\r\n\r\n"
                )
                .as_bytes(),
            );
            body.extend_from_slice(bytes);
            body.extend_from_slice(b"\r\n");
        }
        body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
        let req = Request::builder()
            .method("POST")
            .uri("/api/finance/statements")
            .header(
                "content-type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap();
        reply(self.app.clone().oneshot(req).await.unwrap()).await
    }

    /// Uploads one file and returns the id of its Statement.
    pub async fn upload_one(&self, name: &str, bytes: &[u8]) -> i64 {
        let reply = self.upload(&[(name, bytes)]).await;
        assert_eq!(reply.status, 200, "{}", reply.text);
        reply.json[0]["id"].as_i64().unwrap()
    }

    /// Confirms with every section accepting its proposal.
    pub async fn confirm_proposed(&self, id: i64, sections: usize) -> Reply {
        let body = serde_json::json!({"sections": vec![serde_json::json!({}); sections]});
        self.post(&format!("/statements/{id}/confirm"), &body.to_string())
            .await
    }

    pub async fn finish(self) {
        self.db.drop().await;
    }
}

async fn reply(res: axum::response::Response) -> Reply {
    let status = res.status().as_u16();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    Reply {
        status,
        text: String::from_utf8_lossy(&bytes).into_owned(),
        json: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    }
}
