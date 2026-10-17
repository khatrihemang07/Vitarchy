//! A [`Period`]: the local Days between two dates in a time zone, each with its
//! exact UTC bounds. Pure; a Day runs midnight to midnight local time (GLOSSARY).

use chrono::{DateTime, Days, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;

/// The longest Period accepted, in Days.
pub const MAX_DAYS: usize = 400;

/// Why a Period could not be built. The messages hold no entered values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PeriodError {
    /// `tz` is not an IANA time zone name.
    #[error("the time zone is not a known IANA name such as Europe/London")]
    UnknownTimeZone,
    /// `from` or `to` is not a `YYYY-MM-DD` date.
    #[error("dates must be written YYYY-MM-DD")]
    InvalidDate,
    /// `to` is before `from`.
    #[error("the end date is before the start date")]
    Reversed,
    /// More than [`MAX_DAYS`] Days.
    #[error("a Period can span at most {MAX_DAYS} days")]
    TooLong,
}

impl PeriodError {
    /// Stable machine label for the problem's `code`.
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Self::UnknownTimeZone => "unknown_time_zone",
            Self::InvalidDate => "invalid_date",
            Self::Reversed => "period_reversed",
            Self::TooLong => "period_too_long",
        }
    }
}

/// One local Day: its date and the UTC instants it starts and ends at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Day {
    /// The local calendar date.
    pub date: NaiveDate,
    /// Local midnight that begins the Day, in UTC (inclusive).
    pub start: DateTime<Utc>,
    /// Local midnight that ends the Day, in UTC (exclusive).
    pub end: DateTime<Utc>,
}

/// The consecutive local Days from one date to another, in one time zone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Period {
    tz: Tz,
    days: Vec<Day>,
}

impl Period {
    /// Builds the Period `from..=to` (`YYYY-MM-DD`) in the IANA zone `tz`.
    ///
    /// # Errors
    /// A [`PeriodError`] for an unknown zone, a bad date, reversed dates or more than [`MAX_DAYS`] Days.
    pub fn parse(from: &str, to: &str, tz: &str) -> Result<Self, PeriodError> {
        let tz: Tz = tz.parse().map_err(|_| PeriodError::UnknownTimeZone)?;
        let from = parse_date(from)?;
        let to = parse_date(to)?;
        let count = to.signed_duration_since(from).num_days() + 1;
        if count < 1 {
            return Err(PeriodError::Reversed);
        }
        if usize::try_from(count).map_or(true, |n| n > MAX_DAYS) {
            return Err(PeriodError::TooLong);
        }
        let days = from
            .iter_days()
            .take_while(|date| *date <= to)
            .map(|date| day_of(tz, date))
            .collect();
        Ok(Self { tz, days })
    }

    /// The zone Days are measured in.
    #[must_use]
    pub fn tz(&self) -> Tz {
        self.tz
    }

    /// The Days, earliest first. Never empty.
    #[must_use]
    pub fn days(&self) -> &[Day] {
        &self.days
    }

    /// Start of the first Day: a query's lower bound.
    #[must_use]
    pub fn start(&self) -> DateTime<Utc> {
        self.days.first().map_or(DateTime::UNIX_EPOCH, |d| d.start)
    }

    /// End of the last Day: a query's upper bound.
    #[must_use]
    pub fn end(&self) -> DateTime<Utc> {
        self.days.last().map_or(DateTime::UNIX_EPOCH, |d| d.end)
    }
}

/// The latest local date in `tz` that is over at `now`: the day before today.
#[must_use]
pub fn last_finished_date(tz: Tz, now: DateTime<Utc>) -> NaiveDate {
    let today = now.with_timezone(&tz).date_naive();
    today.pred_opt().unwrap_or(today)
}

fn parse_date(text: &str) -> Result<NaiveDate, PeriodError> {
    NaiveDate::parse_from_str(text, "%Y-%m-%d").map_err(|_| PeriodError::InvalidDate)
}

fn day_of(tz: Tz, date: NaiveDate) -> Day {
    let next = date.checked_add_days(Days::new(1)).unwrap_or(date);
    Day {
        date,
        start: midnight(tz, date),
        end: midnight(tz, next),
    }
}

/// First instant of `date` in `tz`. When a zone skips midnight (a DST jump over
/// it), the Day begins at the first minute that exists.
fn midnight(tz: Tz, date: NaiveDate) -> DateTime<Utc> {
    (0..=24 * 60)
        .find_map(|minute| {
            let local = date.and_time(NaiveTime::MIN) + chrono::Duration::minutes(minute);
            tz.from_local_datetime(&local).earliest()
        })
        .map_or_else(|| date.and_time(NaiveTime::MIN).and_utc(), |t| t.to_utc())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(text: &str) -> DateTime<Utc> {
        text.parse().unwrap()
    }

    #[test]
    fn the_last_finished_date_is_yesterday_in_the_zone() {
        // 23:30 UTC on the 9th is already the 10th in Kolkata, so the 9th is finished there.
        let now = utc("2026-01-09T23:30:00Z");
        assert_eq!(
            last_finished_date(chrono_tz::Asia::Kolkata, now).to_string(),
            "2026-01-09"
        );
        assert_eq!(
            last_finished_date(chrono_tz::UTC, now).to_string(),
            "2026-01-08"
        );
    }

    #[test]
    fn lists_each_local_day_inclusive() {
        let period = Period::parse("2026-03-01", "2026-03-03", "UTC").unwrap();
        let dates: Vec<String> = period.days().iter().map(|d| d.date.to_string()).collect();
        assert_eq!(dates, ["2026-03-01", "2026-03-02", "2026-03-03"]);
    }

    #[test]
    fn a_day_is_midnight_to_midnight_in_the_zone() {
        let period = Period::parse("2026-01-10", "2026-01-10", "Asia/Kolkata").unwrap();
        assert_eq!(period.start(), utc("2026-01-09T18:30:00Z"));
        assert_eq!(period.end(), utc("2026-01-10T18:30:00Z"));
    }

    #[test]
    fn spring_forward_makes_a_23_hour_day_and_fall_back_a_25_hour_day() {
        let spring = Period::parse("2026-03-08", "2026-03-08", "America/New_York").unwrap();
        assert_eq!((spring.end() - spring.start()).num_hours(), 23);
        let fall = Period::parse("2026-11-01", "2026-11-01", "America/New_York").unwrap();
        assert_eq!((fall.end() - fall.start()).num_hours(), 25);
    }

    #[test]
    fn a_day_whose_midnight_is_skipped_starts_when_the_clock_resumes() {
        // Sao Paulo skipped 00:00 on 2018-11-04 (clocks went 00:00 -> 01:00).
        let period = Period::parse("2018-11-04", "2018-11-04", "America/Sao_Paulo").unwrap();
        assert_eq!(period.start(), utc("2018-11-04T03:00:00Z"));
    }

    #[test]
    fn consecutive_days_share_their_boundary() {
        let period = Period::parse("2026-03-07", "2026-03-10", "America/New_York").unwrap();
        for pair in period.days().windows(2) {
            assert_eq!(pair[0].end, pair[1].start);
        }
    }

    #[test]
    fn rejects_bad_input_with_a_stable_code() {
        let cases = [
            (
                ("2026-01-01", "2026-01-02", "Mars/Base"),
                "unknown_time_zone",
            ),
            (("2026-01-01", "2026-01-02", ""), "unknown_time_zone"),
            (("2026-13-01", "2026-01-02", "UTC"), "invalid_date"),
            (("", "2026-01-02", "UTC"), "invalid_date"),
            (("2026-01-03", "2026-01-02", "UTC"), "period_reversed"),
            (("2025-01-01", "2026-12-31", "UTC"), "period_too_long"),
        ];
        for ((from, to, tz), code) in cases {
            let error = Period::parse(from, to, tz).unwrap_err();
            assert_eq!(error.code(), code, "{from} {to} {tz}");
        }
    }

    #[test]
    fn accepts_exactly_the_maximum_length() {
        let from = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let to = from + Days::new(MAX_DAYS as u64 - 1);
        let ok = Period::parse(&from.to_string(), &to.to_string(), "UTC").unwrap();
        assert_eq!(ok.days().len(), MAX_DAYS);
        let too_long = (to + Days::new(1)).to_string();
        assert_eq!(
            Period::parse(&from.to_string(), &too_long, "UTC"),
            Err(PeriodError::TooLong)
        );
    }
}
