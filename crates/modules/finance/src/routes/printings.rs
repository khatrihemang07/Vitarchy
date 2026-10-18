//! `GET /api/finance/transactions/{id}/printings`: every Statement's Printing of a Transaction.

use axum::{
    Json,
    extract::{Path, State},
};
use chrono::{NaiveDate, NaiveTime};
use serde::Serialize;
use utoipa::ToSchema;
use vitarchy_core::AppState;

use crate::{
    error::FinanceError,
    problem::{ProblemBody, Refusal},
    repo::printings_list::{self, PrintingRow},
};

/// One Statement's Printing of a Transaction, as that Statement printed it.
#[derive(Serialize, ToSchema)]
pub struct PrintingItem {
    /// Printing id.
    pub id: i64,
    /// The Statement that printed it.
    pub statement_id: i64,
    /// That Statement's uploaded file name.
    pub file_name: String,
    /// The date as that Statement printed it.
    pub date: NaiveDate,
    /// The local time as that Statement printed it, when it prints one.
    pub time: Option<NaiveTime>,
    /// The description as that Statement printed it.
    pub description: String,
    /// Balance after, when that Statement printed it.
    pub balance_paise: Option<i64>,
    /// A UPI or NEFT reference, when printed.
    pub reference: Option<String>,
    /// True when a Read again of that Statement no longer finds this Printing.
    pub gone: bool,
}

impl From<PrintingRow> for PrintingItem {
    fn from(r: PrintingRow) -> Self {
        Self {
            id: r.id,
            statement_id: r.statement_id,
            file_name: r.file_name,
            date: r.date,
            time: r.time,
            description: r.description,
            balance_paise: r.balance_paise,
            reference: r.reference,
            gone: r.gone,
        }
    }
}

/// A Transaction's Printings, oldest first. The Transaction itself shows the fullest of them.
#[utoipa::path(
    get,
    path = "/transactions/{id}/printings",
    tag = "finance",
    params(("id" = i64, Path, description = "Transaction id")),
    responses(
        (status = 200, description = "Its Printings, in the order they were stored", body = Vec<PrintingItem>),
        (status = 404, description = "No such Transaction (`transaction_not_found`)", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all, fields(transaction_id = id))]
pub async fn list_printings(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<PrintingItem>>, FinanceError> {
    if !printings_list::transaction_exists(&state.pool, id).await? {
        return Err(Refusal::TRANSACTION_NOT_FOUND.into());
    }
    let items: Vec<PrintingItem> = printings_list::of_transaction(&state.pool, id)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
    tracing::info!(printings = items.len(), "printings listed");
    Ok(Json(items))
}
