//! The questions a Report is asked, parsed from the request's query pairs.

use std::collections::BTreeSet;
use std::str::FromStr;

use chrono::NaiveDate;
use chrono_tz::Tz;
use serde::{
    Deserialize,
    de::value::{Error as ValueError, StrDeserializer},
};

use super::{
    filter::Filter,
    grouping::Grouping,
    period::{Period, PeriodError},
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

fn enum_value<T: for<'de> Deserialize<'de>>(
    name: &'static str,
    text: &str,
) -> Result<T, QueryError> {
    T::deserialize(StrDeserializer::<ValueError>::new(text)).map_err(|_| QueryError::Invalid(name))
}

fn numbers(name: &'static str, values: &[&str]) -> Result<BTreeSet<i64>, QueryError> {
    values
        .iter()
        .map(|v| v.parse().map_err(|_| QueryError::Invalid(name)))
        .collect()
}

impl ReportQuery {
    /// Reads `kind`, `from`, `to`, `tz`, repeated `project`/`tag`/`client`, `q`, `group1`
    /// and `group2`. Unknown names are ignored; a repeated single-value name keeps its last.
    pub fn parse(pairs: &[(String, String)]) -> Result<Self, QueryError> {
        let all = |name: &str| -> Vec<&str> {
            pairs
                .iter()
                .filter(|(k, _)| k == name)
                .map(|(_, v)| v.as_str())
                .collect()
        };
        let one = |name: &'static str| all(name).last().copied().filter(|v| !v.is_empty());
        let required = |name: &'static str| one(name).ok_or(QueryError::Missing(name));
        let date = |name: &'static str| {
            NaiveDate::from_str(required(name)?).map_err(|_| QueryError::Invalid(name))
        };
        let tz: Tz = required("tz")?
            .parse()
            .map_err(|_| QueryError::Invalid("tz"))?;
        let kind = enum_value("kind", required("kind")?)?;
        let period = Period::new(kind, date("from")?, date("to")?, tz)?;
        let group1 =
            one("group1").map_or(Ok(Grouping::Description), |v| enum_value("group1", v))?;
        let group2 = one("group2").map(|v| enum_value("group2", v)).transpose()?;
        if group2 == Some(group1) {
            return Err(QueryError::SameGrouping);
        }
        let filter = Filter {
            projects: numbers("project", &all("project"))?,
            clients: numbers("client", &all("client"))?,
            tags: all("tag").into_iter().map(str::to_owned).collect(),
            search: one("q").map(str::to_owned),
        };
        Ok(Self {
            period,
            filter,
            group1,
            group2,
        })
    }
}
