//! `POST /api/finance/statements/read-again` and `.../read-again/confirm`: Read again over
//! every Statement that still has its original.

use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use vitarchy_core::{AppState, RequestId};

use super::read_again::read_one;
use crate::{
    error::FinanceError,
    files::FileStore,
    import::read_again_all::{self, ReadAllAgainCounts},
    problem::ProblemBody,
    reader::StatementReader,
    view::changes::ReadAgainItem,
};

/// Reads every Statement's original again and returns one preview each. Confirmed Statements
/// are only compared; unconfirmed ones are re-read in place, as their own Read again does.
/// A Statement that fails comes back with its error code and does not stop the others.
#[utoipa::path(
    post,
    path = "/statements/read-again",
    tag = "finance",
    responses(
        (status = 200, description = "One item per Statement with a stored original", body = Vec<ReadAgainItem>),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn post_read_all_again(
    State(state): State<AppState>,
    Extension(reader): Extension<Arc<dyn StatementReader>>,
    Extension(files): Extension<FileStore>,
    RequestId(request_id): RequestId,
) -> Result<Json<Vec<ReadAgainItem>>, FinanceError> {
    let rows = read_again_all::with_originals(&state.pool, &files).await?;
    let mut items = Vec::with_capacity(rows.len());
    for row in &rows {
        let outcome = read_one(&state, reader.as_ref(), &files, row, request_id.as_deref()).await;
        let error_code = outcome.as_ref().err().map(FinanceError::code);
        if let Some(code) = error_code {
            tracing::warn!(statement_id = row.id, code, "read again failed");
        }
        items.push(ReadAgainItem {
            statement_id: row.id,
            file_name: row.file_name.clone(),
            preview: outcome.ok(),
            error_code: error_code.map(str::to_owned),
        });
    }
    tracing::info!(statements = items.len(), "read all again previewed");
    Ok(Json(items))
}

/// Applies the Read again of every confirmed Statement that has changes, one at a time under
/// the confirm lock. Statements without changes are left alone; nothing is deleted.
#[utoipa::path(
    post,
    path = "/statements/read-again/confirm",
    tag = "finance",
    responses(
        (status = 200, description = "Applied", body = ReadAllAgainCounts),
        (status = 409, description = "A confirm is running (`confirm_in_progress`)", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn post_read_all_again_confirm(
    State(state): State<AppState>,
    Extension(reader): Extension<Arc<dyn StatementReader>>,
    Extension(files): Extension<FileStore>,
    RequestId(request_id): RequestId,
) -> Result<Json<ReadAllAgainCounts>, FinanceError> {
    Ok(Json(
        read_again_all::apply_all(&state.pool, reader.as_ref(), &files, request_id.as_deref())
            .await?,
    ))
}
