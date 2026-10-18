//! `GET /api/finance/status`: what the Import page shows about the Module.

use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use utoipa::ToSchema;
use vitarchy_core::AppState;

use crate::{
    error::FinanceError,
    reader::StatementReader,
    repo::{
        counts,
        runs::{self, RunRow},
    },
};

/// The kinds of run the status reports, in this order.
const RUN_KINDS: [&str; 3] = ["read", "confirm", "read_again"];

/// A read, confirm or Read again attempt.
#[derive(Serialize, ToSchema)]
pub struct FinanceRunInfo {
    /// `read`, `confirm` or `read_again`.
    pub kind: String,
    /// When it began.
    pub started_at: DateTime<Utc>,
    /// When it ended.
    pub finished_at: DateTime<Utc>,
    /// `success` or `failed`.
    pub outcome: String,
    /// What it did (counts only); empty when it failed.
    #[schema(value_type = Object)]
    pub counts: Value,
    /// Why it failed.
    pub error_code: Option<String>,
    /// The Statement it was about.
    pub statement_id: Option<i64>,
}

impl From<RunRow> for FinanceRunInfo {
    fn from(run: RunRow) -> Self {
        Self {
            kind: run.kind,
            started_at: run.started_at,
            finished_at: run.finished_at,
            outcome: run.outcome,
            counts: run.counts,
            error_code: run.error_code,
            statement_id: run.statement_id,
        }
    }
}

/// Body of `GET /api/finance/status`.
#[derive(Serialize, ToSchema)]
pub struct FinanceStatus {
    /// Accounts stored.
    pub account_count: i64,
    /// Statements waiting to be confirmed.
    pub unconfirmed_statement_count: i64,
    /// Whether the statement-reader helper can be found.
    pub reader_available: bool,
    /// The latest run of each kind that has run, in read, confirm, read again order.
    pub latest_runs: Vec<FinanceRunInfo>,
}

/// Counts, whether the reader is available, and the latest runs.
#[utoipa::path(
    get,
    path = "/status",
    tag = "finance",
    responses((status = 200, description = "Current status", body = FinanceStatus))
)]
#[tracing::instrument(skip_all)]
pub async fn get_finance_status(
    State(state): State<AppState>,
    Extension(reader): Extension<Arc<dyn StatementReader>>,
) -> Result<Json<FinanceStatus>, FinanceError> {
    let pool = &state.pool;
    let mut latest_runs = Vec::new();
    for kind in RUN_KINDS {
        latest_runs.extend(runs::latest(pool, kind).await?.map(FinanceRunInfo::from));
    }
    Ok(Json(FinanceStatus {
        account_count: counts::accounts(pool).await?,
        unconfirmed_statement_count: counts::unconfirmed_statements(pool).await?,
        reader_available: reader.available().await,
        latest_runs,
    }))
}
