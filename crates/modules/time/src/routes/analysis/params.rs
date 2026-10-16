//! The query every analysis route shares: `from`, `to` (local dates) and `tz`.

use serde::Deserialize;
use utoipa::IntoParams;

use crate::{analysis::Period, error::TimeError};

/// A Period asked for by the browser.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PeriodQuery {
    /// First local date, `YYYY-MM-DD`.
    #[serde(default)]
    pub from: String,
    /// Last local date, inclusive, `YYYY-MM-DD`.
    #[serde(default)]
    pub to: String,
    /// IANA time zone the Days are measured in, e.g. `Europe/London`.
    #[serde(default)]
    pub tz: String,
}

impl PeriodQuery {
    /// The Period these parameters name.
    ///
    /// # Errors
    /// [`TimeError::Period`] (a 400 with a `code`) for a bad zone, date or length.
    pub fn period(&self) -> Result<Period, TimeError> {
        Ok(Period::parse(&self.from, &self.to, &self.tz)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query(from: &str, to: &str, tz: &str) -> PeriodQuery {
        PeriodQuery {
            from: from.into(),
            to: to.into(),
            tz: tz.into(),
        }
    }

    #[test]
    fn valid_parameters_name_a_period() {
        let period = query("2026-02-01", "2026-02-07", "Europe/London")
            .period()
            .unwrap();
        assert_eq!(period.days().len(), 7);
    }

    #[test]
    fn missing_parameters_are_a_coded_bad_request() {
        let error = query("", "", "").period().unwrap_err();
        assert_eq!(error.code(), "unknown_time_zone");
    }
}
