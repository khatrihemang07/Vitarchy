//! `POST /api/finance/statements`: upload one or more Statement files.

use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Multipart, State, multipart::MultipartError},
    http::StatusCode,
};
use utoipa::ToSchema;
use vitarchy_core::{AppState, RequestId};

use crate::{
    error::FinanceError,
    files::FileStore,
    import::intake::import_file,
    problem::{ProblemBody, Refusal},
    reader::StatementReader,
    view::{model::StatementPreview, preview},
};

/// The multipart body: one or more `file` parts. Documentation only; the handler reads the
/// stream itself.
#[derive(ToSchema)]
#[allow(dead_code)]
pub struct UploadForm {
    /// The Statement files (PDF, CSV, XLS or XLSX).
    #[schema(format = Binary, value_type = Vec<String>)]
    file: Vec<String>,
}

/// Reads each file into an unconfirmed Statement and returns its preview. A file already
/// stored comes back as that Statement with `duplicate_file` set, without being read again.
#[utoipa::path(
    post,
    path = "/statements",
    tag = "finance",
    request_body(content = UploadForm, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "One preview per file, in upload order", body = Vec<StatementPreview>),
        (status = 400, description = "No files, or unreadable form (`no_files`, `bad_upload`)", body = ProblemBody),
        (status = 413, description = "The upload is too large (`upload_too_large`)", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn post_statements(
    State(state): State<AppState>,
    Extension(reader): Extension<Arc<dyn StatementReader>>,
    Extension(files): Extension<FileStore>,
    RequestId(request_id): RequestId,
    mut form: Multipart,
) -> Result<Json<Vec<StatementPreview>>, FinanceError> {
    let mut previews = Vec::new();
    while let Some(field) = form.next_field().await.map_err(|e| upload_error(&e))? {
        let Some(name) = field.file_name().map(str::to_owned) else {
            continue;
        };
        let bytes = field.bytes().await.map_err(|e| upload_error(&e))?;
        let imported = import_file(
            &state.pool,
            reader.as_ref(),
            &files,
            &name,
            &bytes,
            request_id.as_deref(),
        )
        .await?;
        let row = crate::repo::statements::get(&state.pool, imported.id)
            .await?
            .ok_or(Refusal::STATEMENT_NOT_FOUND)?;
        let mut db = state.pool.acquire().await?;
        previews.push(preview::build(&mut db, &row, imported.duplicate).await?);
    }
    if previews.is_empty() {
        return Err(Refusal::NO_FILES.into());
    }
    tracing::info!(files = previews.len(), "statements uploaded");
    Ok(Json(previews))
}

fn upload_error(error: &MultipartError) -> FinanceError {
    if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
        Refusal::UPLOAD_TOO_LARGE.into()
    } else {
        Refusal::BAD_UPLOAD.into()
    }
}
