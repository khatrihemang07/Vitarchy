//! `GET /api/finance/accounts` and `PATCH /api/finance/accounts/{id}`.

use axum::{
    Json,
    extract::{Path, State},
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use vitarchy_core::{ApiError, ApiJson, AppState, RequestId};

use crate::{
    error::FinanceError,
    linking,
    problem::{ProblemBody, Refusal},
    repo::{
        account_stats::{self, AccountStats},
        accounts,
    },
};

/// An Account with its figures.
#[derive(Serialize, ToSchema)]
pub struct AccountSummary {
    /// Account id.
    pub id: i64,
    /// The issuer's name.
    pub issuer: String,
    /// `savings`, `credit_card` or `loan`.
    pub kind: String,
    /// The number as printed.
    pub mask: Option<String>,
    /// The name the person sees.
    pub name: String,
    /// True when the person hid it: its Transactions leave the Transactions page and it takes
    /// no part in Transfers.
    pub hidden: bool,
    /// Balance after the latest-dated Transaction that printed one.
    pub latest_balance_paise: Option<i64>,
    /// Transactions stored (rows a Read again no longer finds are not counted).
    pub transaction_count: i64,
    /// Date of the earliest Transaction.
    pub first_date: Option<NaiveDate>,
    /// Date of the latest Transaction.
    pub last_date: Option<NaiveDate>,
    /// Confirmed Statements that list this Account.
    pub statement_count: i64,
}

impl From<AccountStats> for AccountSummary {
    fn from(a: AccountStats) -> Self {
        Self {
            id: a.id,
            issuer: a.issuer,
            kind: a.kind,
            mask: a.mask,
            name: a.name,
            hidden: a.hidden,
            latest_balance_paise: a.latest_balance_paise,
            transaction_count: a.transaction_count,
            first_date: a.first_date,
            last_date: a.last_date,
            statement_count: a.statement_count,
        }
    }
}

/// Body of `PATCH /api/finance/accounts/{id}`: whichever of the fields are given change.
#[derive(Deserialize, ToSchema)]
pub struct UpdateAccount {
    /// The new name; trimmed, not empty.
    pub name: Option<String>,
    /// Hide the Account (its Transactions leave the Transactions page and it takes no part in
    /// Transfers) or show it again.
    pub hidden: Option<bool>,
}

/// Every Account with its figures, oldest first.
#[utoipa::path(
    get,
    path = "/accounts",
    tag = "finance",
    responses((status = 200, description = "The Accounts", body = Vec<AccountSummary>))
)]
#[tracing::instrument(skip_all)]
pub async fn list_accounts(
    State(state): State<AppState>,
) -> Result<Json<Vec<AccountSummary>>, FinanceError> {
    let rows = account_stats::list(&state.pool).await?;
    tracing::info!(count = rows.len(), "accounts listed");
    Ok(Json(rows.into_iter().map(AccountSummary::from).collect()))
}

/// Renames an Account, hides it or shows it again.
#[utoipa::path(
    patch,
    path = "/accounts/{id}",
    tag = "finance",
    params(("id" = i64, Path, description = "Account id")),
    request_body = UpdateAccount,
    responses(
        (status = 200, description = "Updated", body = AccountSummary),
        (status = 400, description = "Empty name (`empty_name`) or nothing to change (`nothing_to_change`)", body = ProblemBody),
        (status = 404, description = "No such Account"),
    )
)]
#[tracing::instrument(skip_all, fields(account_id = id))]
pub async fn patch_account(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    RequestId(request_id): RequestId,
    ApiJson(body): ApiJson<UpdateAccount>,
) -> Result<Json<AccountSummary>, FinanceError> {
    let name = body.name.as_deref().map(str::trim);
    if name.is_some_and(str::is_empty) {
        return Err(Refusal::EMPTY_NAME.into());
    }
    if name.is_none() && body.hidden.is_none() {
        return Err(Refusal::NOTHING_TO_CHANGE.into());
    }
    let missing = || ApiError::NotFound("no such Account".to_owned());
    if let Some(name) = name
        && !accounts::rename(&state.pool, id, name).await?
    {
        return Err(missing().into());
    }
    if let Some(hidden) = body.hidden
        && !linking::set_hidden(&state.pool, id, hidden, request_id.as_deref()).await?
    {
        return Err(missing().into());
    }
    let row = account_stats::get(&state.pool, id)
        .await?
        .ok_or_else(missing)?;
    tracing::info!(renamed = name.is_some(), "account updated");
    Ok(Json(row.into()))
}
