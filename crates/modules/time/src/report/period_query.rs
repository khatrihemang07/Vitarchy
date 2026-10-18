//! Reading a Period (`kind`, `from`, `to`, `tz`) from a request's query pairs. Shared by
//! every endpoint that takes a Period, so they all refuse the same things the same way.

use std::str::FromStr;

use chrono::NaiveDate;
use chrono_tz::Tz;
use serde::{
    Deserialize,
    de::value::{Error as ValueError, StrDeserializer},
};

use super::{period::Period, query::QueryError};

/// The query pairs of a request, with the lookups the endpoints share.
#[derive(Debug, Clone, Copy)]
pub struct Pairs<'a>(pub &'a [(String, String)]);

impl<'a> Pairs<'a> {
    /// Every value given for `name`, in order.
    #[must_use]
    pub fn all(&self, name: &str) -> Vec<&'a str> {
        self.0
            .iter()
            .filter(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
            .collect()
    }

    /// The last non-empty value given for `name`.
    #[must_use]
    pub fn one(&self, name: &str) -> Option<&'a str> {
        self.all(name).last().copied().filter(|v| !v.is_empty())
    }

    /// Like [`Self::one`], but a missing value is an error naming `name`.
    pub fn required(&self, name: &'static str) -> Result<&'a str, QueryError> {
        self.one(name).ok_or(QueryError::Missing(name))
    }
}

/// Reads `text` as the serialized name of an enum; the error names the parameter only.
pub fn enum_value<T: for<'de> Deserialize<'de>>(
    name: &'static str,
    text: &str,
) -> Result<T, QueryError> {
    T::deserialize(StrDeserializer::<ValueError>::new(text)).map_err(|_| QueryError::Invalid(name))
}

/// Reads and validates `kind`, `from`, `to` and `tz`. Errors name the parameter, never its value.
pub fn parse_period(pairs: &Pairs<'_>) -> Result<Period, QueryError> {
    let date = |name: &'static str| {
        NaiveDate::from_str(pairs.required(name)?).map_err(|_| QueryError::Invalid(name))
    };
    let tz: Tz = pairs
        .required("tz")?
        .parse()
        .map_err(|_| QueryError::Invalid("tz"))?;
    let kind = enum_value("kind", pairs.required("kind")?)?;
    Ok(Period::new(kind, date("from")?, date("to")?, tz)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::period::{PeriodError, PeriodKind};

    fn pairs(raw: &[(&str, &str)]) -> Vec<(String, String)> {
        raw.iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect()
    }

    fn parse(raw: &[(&str, &str)]) -> Result<Period, QueryError> {
        parse_period(&Pairs(&pairs(raw)))
    }

    const WEEK: [(&str, &str); 4] = [
        ("kind", "week"),
        ("from", "2026-10-12"),
        ("to", "2026-10-18"),
        ("tz", "Europe/Berlin"),
    ];

    #[test]
    fn reads_a_week_in_a_time_zone() {
        let period = parse(&WEEK).unwrap();
        assert_eq!(period.kind, PeriodKind::Week);
        assert_eq!(period.to.to_string(), "2026-10-18");
        assert_eq!(period.tz.name(), "Europe/Berlin");
    }

    #[test]
    fn a_missing_parameter_is_named() {
        for gone in ["kind", "from", "to", "tz"] {
            let raw: Vec<_> = WEEK.iter().copied().filter(|(k, _)| *k != gone).collect();
            let want = QueryError::Missing(WEEK.iter().find(|(k, _)| *k == gone).unwrap().0);
            assert_eq!(parse(&raw).unwrap_err(), want);
        }
    }

    #[test]
    fn a_bad_value_names_the_parameter_but_not_the_value() {
        let with = |name: &str, value: &'static str| -> Vec<(&str, &str)> {
            WEEK.iter()
                .map(|(k, v)| (*k, if *k == name { value } else { *v }))
                .collect()
        };
        for (name, value) in [("kind", "year"), ("from", "10/12"), ("tz", "Mars/Base")] {
            let err = parse(&with(name, value)).unwrap_err();
            assert!(matches!(err, QueryError::Invalid(n) if n == name));
            assert!(!err.to_string().contains(value));
        }
    }

    #[test]
    fn the_kind_must_fit_the_dates_and_an_empty_value_counts_as_missing() {
        let day = parse(&[
            ("kind", "day"),
            ("from", "2026-10-12"),
            ("to", "2026-10-18"),
            ("tz", "UTC"),
        ]);
        assert_eq!(
            day.unwrap_err(),
            QueryError::Period(PeriodError::KindMismatch)
        );
        let empty = parse(&[
            ("kind", ""),
            ("from", "2026-10-12"),
            ("to", "2026-10-12"),
            ("tz", "UTC"),
        ]);
        assert_eq!(empty.unwrap_err(), QueryError::Missing("kind"));
    }

    #[test]
    fn a_repeated_name_keeps_its_last_value() {
        let owned = pairs(&[("a", "1"), ("a", "2"), ("b", "")]);
        let pairs = Pairs(&owned);
        assert_eq!(pairs.one("a"), Some("2"));
        assert_eq!(pairs.one("b"), None);
        assert_eq!(pairs.all("a"), ["1", "2"]);
    }
}
