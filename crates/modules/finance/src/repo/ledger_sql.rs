//! The SQL the Transactions page and the Overview share, so both read the same rows the same
//! way: hidden Accounts out, Transfers joined in, one filter. The fragments use the positional
//! parameters `$1` Account ids (empty = all), `$2` from, `$3` to, `$4` search pattern; callers
//! bind them only through [`bind_filter`], so the order lives here and cannot drift.

use sqlx::{Postgres, postgres::PgArguments, query::QueryAs};

use crate::ledger::{Filter, Flow};

/// Binds `$1` to `$4` of the fragments in this file from `filter`. Bind any further
/// parameters (`$5` onward) after calling it.
pub fn bind_filter<'q, O>(
    query: QueryAs<'q, Postgres, O, PgArguments>,
    filter: &'q Filter,
) -> QueryAs<'q, Postgres, O, PgArguments> {
    query
        .bind(&filter.account_ids)
        .bind(filter.from)
        .bind(filter.to)
        .bind(filter.pattern.as_deref())
}

/// Each Transaction's Transfer (`members`) and each Transfer's debit, credited sum and the
/// credit carrying its Extra (`moved`). The last credit by date carries it, so the Extra lands
/// on the day the surplus arrived.
pub const WITH_TRANSFERS: &str = "WITH members AS (
    SELECT debit_id AS tx_id, id AS transfer_id FROM finance.transfers
    UNION ALL
    SELECT credit_id, transfer_id FROM finance.transfer_credits
), moved AS (
    SELECT x.id AS transfer_id, d.amount_paise AS debit_paise,
           sum(r.amount_paise)::bigint AS credited_paise,
           (array_agg(r.id ORDER BY r.date DESC, r.id DESC))[1] AS last_credit_id
    FROM finance.transfers x
    JOIN finance.transactions d ON d.id = x.debit_id
    JOIN finance.transfer_credits c ON c.transfer_id = x.id
    JOIN finance.transactions r ON r.id = c.credit_id
    GROUP BY x.id, d.amount_paise
)";

/// Rows of hidden Accounts never show; Transfer facts ride along (`m`, `tr`).
pub const FROM: &str = "FROM finance.transactions t
JOIN finance.accounts a ON a.id = t.account_id AND NOT a.hidden
LEFT JOIN members m ON m.tx_id = t.id
LEFT JOIN moved tr ON tr.transfer_id = m.transfer_id";

/// The description search also looks at every Printing, so a shorter print still finds the row.
const SEARCH: &str = "($4::text IS NULL OR t.description ILIKE $4 OR EXISTS (
    SELECT 1 FROM finance.printings g WHERE g.transaction_id = t.id AND g.description ILIKE $4))";

/// What each direction keeps. A Transfer's debit is Money out only for what is still to arrive
/// (its remainder), and a Transfer's credit is Money in only when it carries an Extra: the same
/// rules as `transfers::figures`, which the Overview sums use.
const fn flow_condition(flow: Option<Flow>) -> &'static str {
    match flow {
        None => "TRUE",
        Some(Flow::Transfer) => "m.transfer_id IS NOT NULL",
        Some(Flow::Out) => {
            "(t.direction = 'debit' AND (m.transfer_id IS NULL OR tr.debit_paise > tr.credited_paise))"
        }
        Some(Flow::In) => {
            "(t.direction = 'credit' AND (m.transfer_id IS NULL
                OR (t.id = tr.last_credit_id AND tr.credited_paise > tr.debit_paise)))"
        }
    }
}

/// The `WHERE` of a filter (see the module docs for the parameters).
#[must_use]
pub fn where_clause(flow: Option<Flow>) -> String {
    format!(
        "WHERE (cardinality($1::bigint[]) = 0 OR t.account_id = ANY($1))
  AND ($2::date IS NULL OR t.date >= $2) AND ($3::date IS NULL OR t.date <= $3)
  AND {SEARCH} AND {}",
        flow_condition(flow)
    )
}
