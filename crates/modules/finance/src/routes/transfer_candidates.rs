//! `GET /api/finance/transactions/{id}/transfer-candidates`: what a Transaction could be
//! linked to by hand.

use axum::{
    Json,
    extract::{Path, State},
};
use chrono::NaiveDate;
use serde::Serialize;
use utoipa::ToSchema;
use vitarchy_core::AppState;

use crate::{
    error::FinanceError, linking, problem::ProblemBody, reader::contract::Direction,
    repo::transfer_facts::Facts,
};

/// A Transaction that could be the other side of a Transfer.
#[derive(Serialize, ToSchema)]
pub struct TransferCandidate {
    /// Transaction id.
    pub id: i64,
    /// Its Account.
    pub account_id: i64,
    /// The printed date.
    pub date: NaiveDate,
    /// The shown description.
    pub description: String,
    /// Amount in paise, above 0.
    pub amount_paise: i64,
    /// Which way the money moved (the opposite of the Transaction asked about).
    pub direction: Direction,
}

impl From<Facts> for TransferCandidate {
    fn from(f: Facts) -> Self {
        Self {
            id: f.id,
            account_id: f.account_id,
            date: f.date,
            description: f.description,
            amount_paise: f.amount_paise,
            direction: f.direction,
        }
    }
}

/// The Transactions the given one could be linked to as a Transfer: the other direction, on
/// another Account that is not hidden, within 7 days, in no Transfer yet, and with an amount
/// that fits (a credit may be less than the debit, or up to 50 rupees more). Nearest first.
#[utoipa::path(
    get,
    path = "/transactions/{id}/transfer-candidates",
    tag = "finance",
    params(("id" = i64, Path, description = "Transaction id")),
    responses(
        (status = 200, description = "The candidates", body = Vec<TransferCandidate>),
        (status = 404, description = "No such Transaction (`transaction_not_found`)", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all, fields(transaction_id = id))]
pub async fn list_transfer_candidates(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<TransferCandidate>>, FinanceError> {
    let found = linking::candidates_for(&state.pool, id).await?;
    tracing::info!(count = found.len(), "transfer candidates listed");
    Ok(Json(found.into_iter().map(Into::into).collect()))
}
