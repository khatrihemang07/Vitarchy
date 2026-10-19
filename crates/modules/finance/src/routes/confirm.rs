//! `POST /api/finance/statements/{id}/confirm`.

use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use utoipa::ToSchema;
use vitarchy_core::{ApiJson, AppState, RequestId};

use crate::{
    error::FinanceError,
    import::{
        choice::{Choice, SectionChoice},
        confirm::{ConfirmCounts, confirm},
    },
    problem::ProblemBody,
};

/// Body of the confirm: one choice per Account listed in the Statement, in printed order.
/// A payment app's history has one per instrument.
#[derive(Deserialize, ToSchema)]
pub struct ConfirmRequest {
    /// The choices.
    #[serde(default)]
    pub sections: Vec<SectionChoice>,
}

/// Writes the Statement's new rows to the chosen Accounts. Rows already stored gain a
/// Printing instead; a failed Check does not stop a confirm.
#[utoipa::path(
    post,
    path = "/statements/{id}/confirm",
    tag = "finance",
    params(("id" = i64, Path, description = "Statement id")),
    request_body = ConfirmRequest,
    responses(
        (status = 200, description = "Confirmed", body = ConfirmCounts),
        (status = 400, description = "A choice is missing or invalid (`sections_mismatch`, `invalid_section`, `new_account_required`, `invalid_new_account`, `account_ambiguous`)", body = ProblemBody),
        (status = 404, description = "No such Statement or Account (`statement_not_found`, `account_not_found`)", body = ProblemBody),
        (status = 409, description = "Not readable, already confirmed, or another confirm is running (`statement_not_readable`, `statement_confirmed`, `confirm_in_progress`)", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all, fields(statement_id = id))]
pub async fn post_confirm(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    RequestId(request_id): RequestId,
    ApiJson(body): ApiJson<ConfirmRequest>,
) -> Result<Json<ConfirmCounts>, FinanceError> {
    let choices = body
        .sections
        .iter()
        .map(Choice::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(
        confirm(&state.pool, id, &choices, request_id.as_deref()).await?,
    ))
}
