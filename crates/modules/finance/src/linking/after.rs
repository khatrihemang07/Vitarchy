//! Running detection after something changed. A failure is logged and never undoes the
//! change that caused it: the next run (or the person) can link what this one missed.

use chrono::NaiveDate;
use sqlx::PgPool;

use super::{run::run, trigger::Cause};
use crate::{error::FinanceError, repo::transaction_dates};

/// Where a change could have made new Transfers possible.
enum Scope {
    /// Anywhere.
    Everything,
    /// Near these dates.
    Near(NaiveDate, NaiveDate),
    /// Nowhere: the change touched no Transaction.
    Nothing,
}

impl From<Option<(NaiveDate, NaiveDate)>> for Scope {
    fn from(span: Option<(NaiveDate, NaiveDate)>) -> Self {
        span.map_or(Self::Nothing, |(from, to)| Self::Near(from, to))
    }
}

/// After Statement `statement_id` was confirmed or read again: look near its dates.
pub async fn after_statement(pool: &PgPool, cause: Cause<'_>, statement_id: i64) {
    let span = transaction_dates::of_statement(pool, statement_id).await;
    settle(pool, cause, span.map(Scope::from).map_err(Into::into)).await;
}

/// After these Accounts changed (merged, shown): look near all of their dates.
pub async fn after_accounts(pool: &PgPool, cause: Cause<'_>, account_ids: &[i64]) {
    let span = transaction_dates::of_accounts(pool, account_ids).await;
    settle(pool, cause, span.map(Scope::from).map_err(Into::into)).await;
}

/// After a change that could touch any date: look at everything.
pub async fn after_everything(pool: &PgPool, cause: Cause<'_>) {
    settle(pool, cause, Ok(Scope::Everything)).await;
}

async fn settle(pool: &PgPool, cause: Cause<'_>, scope: Result<Scope, FinanceError>) {
    let outcome = match scope {
        Ok(Scope::Nothing) => return,
        Ok(Scope::Everything) => run(pool, cause, None).await.map(drop),
        Ok(Scope::Near(from, to)) => run(pool, cause, Some((from, to))).await.map(drop),
        Err(error) => Err(error),
    };
    if let Err(error) = outcome {
        tracing::warn!(
            trigger = cause.trigger.as_str(),
            request_id = cause.request_id,
            code = error.code(),
            "transfer detection failed"
        );
    }
}
