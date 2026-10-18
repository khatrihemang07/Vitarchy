//! The questions a Report is asked, parsed from the request's query pairs.

use std::collections::BTreeSet;

use super::{
    filter::Filter,
    grouping::Grouping,
    period::{Period, PeriodError},
    period_query::{Pairs, enum_value, parse_period},
};

/// Why a query was refused. Messages name the parameter, never its value.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum QueryError {
    /// A required parameter is absent.
    #[error("`{0}` is required")]
    Missing(&'static str),
    /// A parameter has a value that cannot be read.
    #[error("`{0}` is not valid")]
    Invalid(&'static str),
    /// The two groupings are the same.
    #[error("`group2` must differ from `group1`")]
    SameGrouping,
    /// The Period is not acceptable.
    #[error(transparent)]
    Period(#[from] PeriodError),
}

/// A validated Report request.
#[derive(Debug, Clone)]
pub struct ReportQuery {
    /// The Days to cover.
    pub period: Period,
    /// Which entries count.
    pub filter: Filter,
    /// First-level grouping.
    pub group1: Grouping,
    /// Optional second-level grouping.
    pub group2: Option<Grouping>,
}

impl ReportQuery {
    /// The groupings in order, one or two.
    #[must_use]
    pub fn groupings(&self) -> Vec<Grouping> {
        std::iter::once(self.group1).chain(self.group2).collect()
    }
}

/// Toggl ids: positive whole numbers. Empty values (`project=`) are skipped, like an empty tag.
fn numbers(name: &'static str, values: &[&str]) -> Result<BTreeSet<i64>, QueryError> {
    values
        .iter()
        .filter(|v| !v.is_empty())
        .map(|v| match v.parse::<i64>() {
            Ok(id) if id > 0 => Ok(id),
            _ => Err(QueryError::Invalid(name)),
        })
        .collect()
}

impl ReportQuery {
    /// Reads `kind`, `from`, `to`, `tz`, repeated `project`/`tag`/`client`, `q`, `group1`
    /// and `group2`. Unknown names and empty list values are ignored; a repeated single-value name keeps its last; `q` is trimmed.
    pub fn parse(pairs: &[(String, String)]) -> Result<Self, QueryError> {
        let pairs = Pairs(pairs);
        let (all, one) = (|name| pairs.all(name), |name| pairs.one(name));
        let period = parse_period(&pairs)?;
        let group1 =
            one("group1").map_or(Ok(Grouping::Description), |v| enum_value("group1", v))?;
        let group2 = one("group2").map(|v| enum_value("group2", v)).transpose()?;
        if group2 == Some(group1) {
            return Err(QueryError::SameGrouping);
        }
        let filter = Filter {
            projects: numbers("project", &all("project"))?,
            clients: numbers("client", &all("client"))?,
            tags: all("tag")
                .into_iter()
                .filter(|t| !t.is_empty())
                .map(str::to_owned)
                .collect(),
            // A blank search would match every description with a space in it.
            search: one("q")
                .map(str::trim)
                .filter(|q| !q.is_empty())
                .map(str::to_owned),
        };
        Ok(Self {
            period,
            filter,
            group1,
            group2,
        })
    }
}
