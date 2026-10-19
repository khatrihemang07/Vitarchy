//! Reading the query of `GET /transactions` and `GET /overview`.

use chrono::NaiveDate;

use super::like::contains_pattern;

/// Which Transactions to keep by what they mean for the person's money.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    /// Money in: credits outside Transfers, and credits carrying an Extra.
    In,
    /// Money out: debits outside Transfers, and debits of partial Transfers.
    Out,
    /// Either side of a Transfer.
    Transfer,
}

/// How the rows are ordered; ties break by id the same way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sort {
    /// Newest first.
    #[default]
    DateDesc,
    /// Oldest first.
    DateAsc,
    /// Largest first.
    AmountDesc,
    /// Smallest first.
    AmountAsc,
}

impl Sort {
    /// The `ORDER BY` terms of this order (fixed text, never built from the query).
    #[must_use]
    pub const fn order_by(self) -> &'static str {
        match self {
            Self::DateDesc => "t.date DESC, t.id DESC",
            Self::DateAsc => "t.date ASC, t.id ASC",
            Self::AmountDesc => "t.amount_paise DESC, t.id DESC",
            Self::AmountAsc => "t.amount_paise ASC, t.id ASC",
        }
    }
}

/// Which Transactions count. Hidden Accounts are always left out by the SQL.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filter {
    /// Only these Accounts; empty means all.
    pub account_ids: Vec<i64>,
    /// On or after this date.
    pub from: Option<NaiveDate>,
    /// On or before this date.
    pub to: Option<NaiveDate>,
    /// The `ILIKE` pattern of the description search.
    pub pattern: Option<String>,
    /// Money in, Money out or Transfers only.
    pub flow: Option<Flow>,
}

/// A parsed request: the filter plus the order and page.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LedgerQuery {
    /// What to keep.
    pub filter: Filter,
    /// How to order.
    pub sort: Sort,
    /// Rows per page, when asked.
    pub limit: Option<i64>,
    /// Rows to skip, when asked.
    pub offset: Option<i64>,
}

/// Why a query was refused. Never quotes the value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("invalid query parameter `{0}`")]
pub struct QueryError(pub &'static str);

fn single<'a>(pairs: &'a [(String, String)], name: &str) -> Option<&'a str> {
    pairs
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_str())
}

fn number(pairs: &[(String, String)], name: &'static str) -> Result<Option<i64>, QueryError> {
    single(pairs, name)
        .map(|v| v.parse().map_err(|_| QueryError(name)))
        .transpose()
}

fn date(pairs: &[(String, String)], name: &'static str) -> Result<Option<NaiveDate>, QueryError> {
    single(pairs, name)
        .map(|v| NaiveDate::parse_from_str(v, "%Y-%m-%d").map_err(|_| QueryError(name)))
        .transpose()
}

fn flow(pairs: &[(String, String)]) -> Result<Option<Flow>, QueryError> {
    match single(pairs, "direction") {
        None => Ok(None),
        Some("in") => Ok(Some(Flow::In)),
        Some("out") => Ok(Some(Flow::Out)),
        Some("transfer") => Ok(Some(Flow::Transfer)),
        Some(_) => Err(QueryError("direction")),
    }
}

fn sort(pairs: &[(String, String)]) -> Result<Sort, QueryError> {
    match single(pairs, "sort") {
        None | Some("date_desc") => Ok(Sort::DateDesc),
        Some("date_asc") => Ok(Sort::DateAsc),
        Some("amount_desc") => Ok(Sort::AmountDesc),
        Some("amount_asc") => Ok(Sort::AmountAsc),
        Some(_) => Err(QueryError("sort")),
    }
}

impl LedgerQuery {
    /// Reads a raw query string (`a=1&a=2`); see [`Self::parse`].
    ///
    /// # Errors
    /// As [`Self::parse`].
    pub fn from_raw(raw: Option<&str>) -> Result<Self, QueryError> {
        let pairs: Vec<(String, String)> =
            url::form_urlencoded::parse(raw.unwrap_or_default().as_bytes())
                .into_owned()
                .collect();
        Self::parse(&pairs)
    }

    /// Reads the query pairs (`account_id` may repeat). Unknown names are ignored; a blank
    /// `q` is no search; `from` after `to` is refused.
    ///
    /// # Errors
    /// A value is malformed or the range is backwards.
    pub fn parse(pairs: &[(String, String)]) -> Result<Self, QueryError> {
        let account_ids = pairs
            .iter()
            .filter(|(key, _)| key == "account_id")
            .map(|(_, v)| v.parse().map_err(|_| QueryError("account_id")))
            .collect::<Result<_, _>>()?;
        let (from, to) = (date(pairs, "from")?, date(pairs, "to")?);
        if from.zip(to).is_some_and(|(from, to)| from > to) {
            return Err(QueryError("to"));
        }
        let pattern = single(pairs, "q")
            .map(str::trim)
            .filter(|q| !q.is_empty())
            .map(contains_pattern);
        Ok(Self {
            filter: Filter {
                account_ids,
                from,
                to,
                pattern,
                flow: flow(pairs)?,
            },
            sort: sort(pairs)?,
            limit: number(pairs, "limit")?,
            offset: number(pairs, "offset")?,
        })
    }
}
