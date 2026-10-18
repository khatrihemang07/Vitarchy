//! `POST /api/finance/statements/{id}/read-again` and `.../read-again/confirm`.

use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Path, State},
};
use vitarchy_core::{AppState, RequestId};

use super::statements::load;
use crate::{
    error::FinanceError,
    files::FileStore,
    import::{
        read_again::{self, ReadAgainCounts},
        retry,
    },
    problem::ProblemBody,
    reader::StatementReader,
    repo::statements::StatementRow,
    view::{
        changes::{ReadAgainPreview, of_diff},
        preview,
    },
};

/// Reads the stored original again. An unconfirmed Statement is simply re-read with the
/// passwords saved now (this is how a locked file opens). A confirmed one is compared with
/// what is stored and nothing is written until `read-again/confirm`.
#[utoipa::path(
    post,
    path = "/statements/{id}/read-again",
    tag = "finance",
    params(("id" = i64, Path, description = "Statement id")),
    responses(
        (status = 200, description = "The Statement and what would change", body = ReadAgainPreview),
        (status = 404, description = "No such Statement (`statement_not_found`)", body = ProblemBody),
        (status = 409, description = "The engine no longer reads the file as before (`statement_not_readable`, `statement_sections_changed`)", body = ProblemBody),
        (status = 502, description = "The statement reader failed", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all, fields(statement_id = id))]
pub async fn post_read_again(
    State(state): State<AppState>,
    Extension(reader): Extension<Arc<dyn StatementReader>>,
    Extension(files): Extension<FileStore>,
    RequestId(request_id): RequestId,
    Path(id): Path<i64>,
) -> Result<Json<ReadAgainPreview>, FinanceError> {
    let row = load(&state, id).await?;
    Ok(Json(
        read_one(&state, reader.as_ref(), &files, &row, request_id.as_deref()).await?,
    ))
}

/// Read again for one Statement, as the single and the all-Statements routes share it.
pub(super) async fn read_one(
    state: &AppState,
    reader: &dyn StatementReader,
    files: &FileStore,
    row: &StatementRow,
    request_id: Option<&str>,
) -> Result<ReadAgainPreview, FinanceError> {
    let (sections, payment_records) = if row.status == "confirmed" {
        let (result, diff) =
            read_again::preview(&state.pool, reader, files, row, request_id).await?;
        of_diff(&diff, &result)
    } else {
        retry::retry(&state.pool, reader, files, row, request_id).await?;
        (vec![], None)
    };
    let row = load(state, row.id).await?;
    let mut db = state.pool.acquire().await?;
    Ok(ReadAgainPreview {
        statement: preview::build(&mut db, &row, false).await?,
        sections,
        payment_records,
    })
}

/// Applies a Read again to a confirmed Statement: adds the new rows, clears marks on rows
/// found again and marks rows no longer found. Nothing is deleted.
#[utoipa::path(
    post,
    path = "/statements/{id}/read-again/confirm",
    tag = "finance",
    params(("id" = i64, Path, description = "Statement id")),
    responses(
        (status = 200, description = "Applied", body = ReadAgainCounts),
        (status = 404, description = "No such Statement (`statement_not_found`)", body = ProblemBody),
        (status = 409, description = "Unconfirmed, unreadable or a confirm is running (`statement_unconfirmed`, `statement_not_readable`, `statement_sections_changed`, `confirm_in_progress`)", body = ProblemBody),
        (status = 502, description = "The statement reader failed", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all, fields(statement_id = id))]
pub async fn post_read_again_confirm(
    State(state): State<AppState>,
    Extension(reader): Extension<Arc<dyn StatementReader>>,
    Extension(files): Extension<FileStore>,
    RequestId(request_id): RequestId,
    Path(id): Path<i64>,
) -> Result<Json<ReadAgainCounts>, FinanceError> {
    Ok(Json(
        read_again::apply(
            &state.pool,
            reader.as_ref(),
            &files,
            id,
            request_id.as_deref(),
        )
        .await?,
    ))
}
