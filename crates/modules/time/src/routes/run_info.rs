//! A Sync or Import attempt as the status response shows it.

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use utoipa::ToSchema;

use crate::repo::runs::RunRow;

/// A Sync or Import attempt.
#[derive(Serialize, ToSchema)]
pub struct RunInfo {
    /// `sync` or `import`.
    pub kind: String,
    /// When it began.
    pub started_at: DateTime<Utc>,
    /// When it ended.
    pub finished_at: DateTime<Utc>,
    /// `success` or `failed`.
    pub outcome: String,
    /// What it did, e.g. `{"inserted": 3}`; empty when it failed.
    #[schema(value_type = Object)]
    pub counts: Value,
    /// Why it failed (`invalid_token`, `no_snapshot`, ...).
    pub error_code: Option<String>,
}

impl From<RunRow> for RunInfo {
    fn from(run: RunRow) -> Self {
        Self {
            kind: run.kind,
            started_at: run.started_at,
            finished_at: run.finished_at,
            outcome: run.outcome,
            counts: run.counts,
            error_code: run.error_code,
        }
    }
}
