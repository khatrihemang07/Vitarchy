//! `GET /api/finance/overview`: Money in, Money out, Net and Transfers over a range, compared
//! with the range just before it, and Money in and out per day.

use axum::{
    Json,
    extract::{RawQuery, State},
};
use chrono::NaiveDate;
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{IntoParams, ToSchema};
use vitarchy_core::AppState;

use crate::{
    error::FinanceError,
    ledger::{self, Filter, LedgerQuery, MAX_DAYS, Totals},
    problem::{ProblemBody, Refusal},
    repo::overview_rows,
};

/// Query of `GET /api/finance/overview`. Documentation only: the handler reads the raw query
/// because axum's `Query` cannot read a repeated name.
#[derive(IntoParams)]
#[into_params(parameter_in = Query)]
#[allow(dead_code)]
pub struct OverviewParams {
    /// First day, `YYYY-MM-DD`.
    from: NaiveDate,
    /// Last day (inclusive), `YYYY-MM-DD`; at most 366 days after `from`.
    to: NaiveDate,
    /// Only these Accounts (repeat the name). Hidden Accounts are never included.
    account_id: Option<Vec<i64>>,
    /// Only descriptions containing this text, ignoring case; any Printing's description counts.
    q: Option<String>,
}

/// One figure of the range and the same figure for the range before it.
#[derive(Serialize, ToSchema)]
pub struct OverviewFigure {
    /// In paise, over the range.
    pub paise: i64,
    /// In paise, over the equally long range just before it.
    pub previous_paise: i64,
}

/// One day's Money in and Money out.
#[derive(Serialize, ToSchema)]
pub struct OverviewDay {
    /// The day.
    pub date: NaiveDate,
    /// Money in, in paise.
    pub money_in_paise: i64,
    /// Money out, in paise.
    pub money_out_paise: i64,
}

/// The Overview of a range. Transfers between the person's own Accounts are left out of Money
/// in and Money out, except a Transfer's Extra (Money in) and a partial Transfer's remainder
/// (Money out).
#[derive(Serialize, ToSchema)]
pub struct FinanceOverview {
    /// Money in.
    pub money_in: OverviewFigure,
    /// Money out.
    pub money_out: OverviewFigure,
    /// Money in minus Money out.
    pub net: OverviewFigure,
    /// What moved between the person's own Accounts.
    pub transfers_total: OverviewFigure,
    /// Every day of the range, days without Transactions included.
    pub daily: Vec<OverviewDay>,
}

impl FinanceOverview {
    /// The figures of `current` set against `previous`.
    #[must_use]
    pub fn of(current: &Totals, previous: &Totals) -> Self {
        let figure = |now: i64, before: i64| OverviewFigure {
            paise: now,
            previous_paise: before,
        };
        let net = |t: &Totals| t.money_in_paise - t.money_out_paise;
        Self {
            money_in: figure(current.money_in_paise, previous.money_in_paise),
            money_out: figure(current.money_out_paise, previous.money_out_paise),
            net: figure(net(current), net(previous)),
            transfers_total: figure(current.transfers_paise, previous.transfers_paise),
            daily: current
                .daily
                .iter()
                .map(|d| OverviewDay {
                    date: d.date,
                    money_in_paise: d.money_in_paise,
                    money_out_paise: d.money_out_paise,
                })
                .collect(),
        }
    }
}

async fn totals(
    pool: &PgPool,
    filter: &Filter,
    (from, to): (NaiveDate, NaiveDate),
) -> Result<Totals, sqlx::Error> {
    let ranged = Filter {
        from: Some(from),
        to: Some(to),
        ..filter.clone()
    };
    let rows = overview_rows::rows(pool, &ranged).await?;
    Ok(ledger::aggregate(&rows, from, to))
}

/// Money in, Money out, Net and Transfers over `from..=to`, each against the range of the same
/// length just before it, and per-day Money in and out. Hidden Accounts are left out.
#[utoipa::path(
    get,
    path = "/overview",
    tag = "finance",
    params(OverviewParams),
    responses(
        (status = 200, description = "The Overview", body = FinanceOverview),
        (status = 400, description = "Invalid range or filter (`invalid_query`)", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn get_overview(
    State(state): State<AppState>,
    RawQuery(raw): RawQuery,
) -> Result<Json<FinanceOverview>, FinanceError> {
    let query = LedgerQuery::from_raw(raw.as_deref()).map_err(|_| Refusal::INVALID_QUERY)?;
    let range = query.filter.from.zip(query.filter.to);
    let (from, to) = range.ok_or(Refusal::INVALID_QUERY)?;
    let previous = ledger::previous_range(from, to).ok_or(Refusal::INVALID_QUERY)?;
    if ledger::days(from, to) > MAX_DAYS {
        return Err(Refusal::INVALID_QUERY.into());
    }
    let current = totals(&state.pool, &query.filter, (from, to)).await?;
    let before = totals(&state.pool, &query.filter, previous).await?;
    tracing::info!(
        days = current.daily.len(),
        accounts = query.filter.account_ids.len(),
        searched = query.filter.pattern.is_some(),
        "overview built"
    );
    Ok(Json(FinanceOverview::of(&current, &before)))
}
