//! `POST /api/finance/transfers` and `DELETE /api/finance/transfers/{id}`: linking Transfers
//! by hand and unlinking them.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use vitarchy_core::{ApiJson, AppState, RequestId};

use super::transfer_info::TransferInfo;
use crate::{
    error::FinanceError,
    linking,
    problem::{ProblemBody, Refusal},
    repo::{transfers, transfers_load},
};

/// Body of `POST /api/finance/transfers`.
#[derive(Deserialize, ToSchema)]
pub struct LinkTransfer {
    /// The Money-out Transaction.
    pub debit_id: i64,
    /// The Money-in Transactions on other Accounts it paid; they may add up to less than the
    /// debit (a partial Transfer) or up to 50 rupees more.
    pub credit_ids: Vec<i64>,
}

/// A Transfer just linked.
#[derive(Serialize, ToSchema)]
pub struct LinkedTransfer {
    /// The Transfer as the debit's row shows it.
    pub transfer: TransferInfo,
    /// The Money-out Transaction.
    pub debit_id: i64,
    /// The Money-in Transactions linked to it.
    pub credit_ids: Vec<i64>,
}

/// Links a Money-out Transaction to Money-in Transactions on other Accounts as a Transfer.
/// Unlike detection it ignores unlinks the person made before.
#[utoipa::path(
    post,
    path = "/transfers",
    tag = "finance",
    request_body = LinkTransfer,
    responses(
        (status = 201, description = "Linked", body = LinkedTransfer),
        (status = 400, description = "The Transactions do not make a Transfer (`transfer_no_credits`, `transfer_wrong_direction`, `transfer_same_account`, `transfer_duplicate_credit`, `transfer_too_much`)", body = ProblemBody),
        (status = 404, description = "No such Transaction (`transaction_not_found`)", body = ProblemBody),
        (status = 409, description = "A hidden Account or a Transaction already in a Transfer (`transfer_hidden_account`, `transfer_already_linked`)", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all, fields(debit_id = body.debit_id))]
pub async fn post_transfer(
    State(state): State<AppState>,
    RequestId(request_id): RequestId,
    ApiJson(body): ApiJson<LinkTransfer>,
) -> Result<(StatusCode, Json<LinkedTransfer>), FinanceError> {
    let parts = linking::link(
        &state.pool,
        body.debit_id,
        &body.credit_ids,
        request_id.as_deref(),
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(LinkedTransfer {
            transfer: TransferInfo::of(&parts, parts.debit_account_id),
            debit_id: body.debit_id,
            credit_ids: body.credit_ids,
        }),
    ))
}

/// Unlinks a Transfer. Detection never links the same pairs again; linking them by hand
/// still works.
#[utoipa::path(
    delete,
    path = "/transfers/{id}",
    tag = "finance",
    params(("id" = i64, Path, description = "Transfer id")),
    responses(
        (status = 204, description = "Unlinked"),
        (status = 404, description = "No such Transfer (`transfer_not_found`)", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all, fields(transfer_id = id))]
pub async fn delete_transfer(
    State(state): State<AppState>,
    RequestId(request_id): RequestId,
    Path(id): Path<i64>,
) -> Result<StatusCode, FinanceError> {
    let credits = transfers_load::credit_ids(&state.pool, id).await?;
    let mut tx = state.pool.begin().await?;
    if !transfers::unlink(&mut tx, id).await? {
        return Err(Refusal::TRANSFER_NOT_FOUND.into());
    }
    tx.commit().await?;
    tracing::info!(credits = credits.len(), request_id, "transfer unlinked");
    Ok(StatusCode::NO_CONTENT)
}
