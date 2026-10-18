//! `GET /api/finance/transactions`: the Transactions table's rows.

use axum::{
    Json,
    extract::{Query, State, rejection::QueryRejection},
};
use chrono::NaiveDate;
use serde::Deserialize;
use utoipa::IntoParams;
use vitarchy_core::AppState;

use super::transaction_items::{TransactionItem, TransactionPage};
use crate::{
    error::FinanceError,
    problem::{ProblemBody, Refusal},
    repo::transaction_page::{self, Filter},
};

/// Rows per page when `limit` is absent.
const DEFAULT_LIMIT: i64 = 100;
/// The most rows one page may hold.
const MAX_LIMIT: i64 = 500;

/// Query of `GET /api/finance/transactions`.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct TransactionParams {
    /// Only this Account.
    account_id: Option<i64>,
    /// Only on or after this date, `YYYY-MM-DD`.
    from: Option<NaiveDate>,
    /// Only on or before this date, `YYYY-MM-DD`.
    to: Option<NaiveDate>,
    /// Rows per page: 1 to 500, default 100.
    limit: Option<i64>,
    /// Rows to skip, default 0.
    offset: Option<i64>,
}

/// Transactions, newest first, with their linked Payment record. Rows a Read again no longer
/// finds are included and flagged.
#[utoipa::path(
    get,
    path = "/transactions",
    tag = "finance",
    params(TransactionParams),
    responses(
        (status = 200, description = "A page of Transactions", body = TransactionPage),
        (status = 400, description = "Invalid filter (`invalid_query`)", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn list_transactions(
    State(state): State<AppState>,
    query: Result<Query<TransactionParams>, QueryRejection>,
) -> Result<Json<TransactionPage>, FinanceError> {
    let Query(p) = query.map_err(|_| Refusal::INVALID_QUERY)?;
    let limit = p.limit.unwrap_or(DEFAULT_LIMIT);
    let offset = p.offset.unwrap_or(0);
    let range_ok = p.from.zip(p.to).is_none_or(|(from, to)| from <= to);
    if !(1..=MAX_LIMIT).contains(&limit) || offset < 0 || !range_ok {
        return Err(Refusal::INVALID_QUERY.into());
    }
    let filter = Filter {
        account_id: p.account_id,
        from: p.from,
        to: p.to,
    };
    let items: Vec<TransactionItem> = transaction_page::page(&state.pool, filter, limit, offset)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
    let total = transaction_page::total(&state.pool, filter).await?;
    tracing::info!(rows = items.len(), total, "transactions listed");
    Ok(Json(TransactionPage { total, items }))
}
