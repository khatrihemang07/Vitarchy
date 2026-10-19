//! `GET /api/finance/transactions`: the Transactions table's rows.

use axum::{
    Json,
    extract::{RawQuery, State},
};
use chrono::NaiveDate;
use utoipa::IntoParams;
use vitarchy_core::AppState;

use super::transaction_items::{TransactionItem, TransactionPage};
use crate::{
    error::FinanceError,
    ledger::LedgerQuery,
    problem::{ProblemBody, Refusal},
    repo::{transaction_page, transfers_load},
};

/// Rows per page when `limit` is absent.
const DEFAULT_LIMIT: i64 = 100;
/// The most rows one page may hold.
const MAX_LIMIT: i64 = 500;

/// How the Transactions are ordered.
#[derive(utoipa::ToSchema)]
#[allow(dead_code)]
#[schema(rename_all = "snake_case")]
pub enum SortParam {
    /// Newest first (the default).
    DateDesc,
    /// Oldest first.
    DateAsc,
    /// Largest first.
    AmountDesc,
    /// Smallest first.
    AmountAsc,
}

/// Which Transactions, by what they mean for the person's money.
#[derive(utoipa::ToSchema)]
#[allow(dead_code)]
#[schema(rename_all = "snake_case")]
pub enum DirectionParam {
    /// Money in: credits outside Transfers, and credits carrying an Extra.
    In,
    /// Money out: debits outside Transfers, and debits of partial Transfers.
    Out,
    /// Either side of a Transfer.
    Transfer,
}

/// Query of `GET /api/finance/transactions`. Documentation only: axum's `Query` cannot read a
/// repeated name (`account_id=1&account_id=2`), so the handler reads the raw query itself.
#[derive(IntoParams)]
#[into_params(parameter_in = Query)]
#[allow(dead_code)]
pub struct TransactionParams {
    /// Only these Accounts (repeat the name). Hidden Accounts are never included.
    account_id: Option<Vec<i64>>,
    /// Only on or after this date, `YYYY-MM-DD`.
    from: Option<NaiveDate>,
    /// Only on or before this date, `YYYY-MM-DD`.
    to: Option<NaiveDate>,
    /// Only descriptions containing this text, ignoring case; any Printing's description counts.
    q: Option<String>,
    /// Only Money in, Money out or Transfers.
    #[param(inline)]
    direction: Option<DirectionParam>,
    /// Order; ties break by id. Default `date_desc`.
    #[param(inline)]
    sort: Option<SortParam>,
    /// Rows per page: 1 to 500, default 100.
    limit: Option<i64>,
    /// Rows to skip, default 0.
    offset: Option<i64>,
}

/// Transactions, newest first unless `sort` says otherwise. Rows a Read again no longer finds are included and flagged.
/// Transactions of hidden Accounts are left out.
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
    RawQuery(raw): RawQuery,
) -> Result<Json<TransactionPage>, FinanceError> {
    let query = LedgerQuery::from_raw(raw.as_deref()).map_err(|_| Refusal::INVALID_QUERY)?;
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT);
    let offset = query.offset.unwrap_or(0);
    if !(1..=MAX_LIMIT).contains(&limit) || offset < 0 {
        return Err(Refusal::INVALID_QUERY.into());
    }
    let filter = &query.filter;
    let rows = transaction_page::page(&state.pool, filter, query.sort, limit, offset).await?;
    let transfer_ids: Vec<i64> = rows.iter().filter_map(|r| r.transfer_id).collect();
    let transfers = transfers_load::parts(&state.pool, &transfer_ids).await?;
    let items: Vec<TransactionItem> = rows
        .into_iter()
        .map(|row| TransactionItem::of(row, &transfers))
        .collect();
    let total = transaction_page::total(&state.pool, filter).await?;
    tracing::info!(
        rows = items.len(),
        total,
        accounts = filter.account_ids.len(),
        searched = filter.pattern.is_some(),
        direction = ?filter.flow,
        sort = ?query.sort,
        "transactions listed"
    );
    Ok(Json(TransactionPage { total, items }))
}
