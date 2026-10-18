//! `GET /api/finance/statements`, `GET` and `DELETE /api/finance/statements/{id}`.

use axum::{
    Extension, Json,
    extract::{Path, Query, State, rejection::QueryRejection},
    http::StatusCode,
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use vitarchy_core::AppState;

use crate::{
    error::FinanceError,
    files::FileStore,
    problem::{ProblemBody, Refusal},
    repo::{statements, statements::StatementRow},
    view::{model::StatementPreview, preview, summary::StatementSummary},
};

/// Which Statements to list.
#[derive(Deserialize, ToSchema, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum StatusFilter {
    /// Waiting for the person to confirm.
    Unconfirmed,
    /// Confirmed.
    Confirmed,
}

/// Query of `GET /api/finance/statements`.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListParams {
    /// Only Statements with this status; all when absent.
    #[param(inline)]
    status: Option<StatusFilter>,
}

pub(super) async fn load(state: &AppState, id: i64) -> Result<StatementRow, FinanceError> {
    Ok(statements::get(&state.pool, id)
        .await?
        .ok_or(Refusal::STATEMENT_NOT_FOUND)?)
}

/// Statements, newest upload first.
#[utoipa::path(
    get,
    path = "/statements",
    tag = "finance",
    params(ListParams),
    responses(
        (status = 200, description = "The Statements", body = Vec<StatementSummary>),
        (status = 400, description = "Unknown status (`invalid_query`)", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn list_statements(
    State(state): State<AppState>,
    query: Result<Query<ListParams>, QueryRejection>,
) -> Result<Json<Vec<StatementSummary>>, FinanceError> {
    let Query(params) = query.map_err(|_| Refusal::INVALID_QUERY)?;
    let rows = match params.status {
        None => statements::list(&state.pool).await?,
        Some(StatusFilter::Unconfirmed) => {
            statements::list_with_status(&state.pool, "unconfirmed").await?
        }
        Some(StatusFilter::Confirmed) => {
            statements::list_with_status(&state.pool, "confirmed").await?
        }
    };
    tracing::info!(count = rows.len(), "statements listed");
    Ok(Json(rows.iter().map(StatementSummary::of).collect()))
}

/// One Statement as a preview.
#[utoipa::path(
    get,
    path = "/statements/{id}",
    tag = "finance",
    params(("id" = i64, Path, description = "Statement id")),
    responses(
        (status = 200, description = "The preview", body = StatementPreview),
        (status = 404, description = "No such Statement (`statement_not_found`)", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all, fields(statement_id = id))]
pub async fn get_statement(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<StatementPreview>, FinanceError> {
    let row = load(&state, id).await?;
    let mut db = state.pool.acquire().await?;
    Ok(Json(preview::build(&mut db, &row, false).await?))
}

/// Discards an unconfirmed Statement and its stored file (unless another Statement uses it).
#[utoipa::path(
    delete,
    path = "/statements/{id}",
    tag = "finance",
    params(("id" = i64, Path, description = "Statement id")),
    responses(
        (status = 204, description = "Discarded"),
        (status = 404, description = "No such Statement (`statement_not_found`)", body = ProblemBody),
        (status = 409, description = "Already confirmed, or its rows are already Transactions' Printings (`statement_confirmed`, `statement_has_rows`)", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all, fields(statement_id = id))]
pub async fn delete_statement(
    State(state): State<AppState>,
    Extension(files): Extension<FileStore>,
    Path(id): Path<i64>,
) -> Result<StatusCode, FinanceError> {
    let row = load(&state, id).await?;
    if !statements::discard(&state.pool, id).await? {
        let refusal = if row.status == "confirmed" {
            Refusal::CONFIRMED
        } else {
            Refusal::HAS_ROWS
        };
        return Err(refusal.into());
    }
    let kept = statements::file_in_use(&state.pool, &row.file_sha256).await?;
    if !kept {
        files.remove(&row.file_sha256).await?;
    }
    tracing::info!(file_removed = !kept, "statement discarded");
    Ok(StatusCode::NO_CONTENT)
}
