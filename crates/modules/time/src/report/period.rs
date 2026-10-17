//! A Period: the Days a Report covers, and the UTC range those Days span.

use chrono::{DateTime, Datelike, Days, Duration, LocalResult, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use serde::Deserialize;
use utoipa::ToSchema;

/// The longest Period a Report accepts, in Days.
pub const MAX_DAYS: i64 = 366;

/// How the Period was chosen; decides what the Previous Period is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, serde::Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PeriodKind {
    /// One Day.
    Day,
    /// Seven Days.
    Week,
    /// A whole calendar month.
    Month,
    /// Any stretch of Days.
    Custom,
}

/// Why a Period was refused. Messages never echo the input.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum PeriodError {
    /// `to` is before `from`.
    #[error("`from` must not be after `to`")]
    Reversed,
    /// More than [`MAX_DAYS`] Days.
    #[error("a Period covers at most 366 Days")]
    TooLong,
    /// The dates do not fit the chosen kind.
    #[error("the dates do not fit the Period kind")]
    KindMismatch,
}

/// Inclusive local dates plus the time zone that gives them their meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Period {
    /// How it was chosen.
    pub kind: PeriodKind,
    /// First Day.
    pub from: NaiveDate,
    /// Last Day (inclusive).
    pub to: NaiveDate,
    /// The person's time zone.
    pub tz: Tz,
}

fn last_of_month(date: NaiveDate) -> NaiveDate {
    let first = date.with_day(1).unwrap_or(date);
    first + chrono::Months::new(1) - Days::new(1)
}

/// Midnight starting `date` in `tz`, as UTC. A midnight skipped by a DST jump
/// starts the Day at the first hour that exists.
pub fn local_midnight(tz: Tz, date: NaiveDate) -> DateTime<Utc> {
    for hour in 0..4 {
        let local = date.and_hms_opt(hour, 0, 0).unwrap_or_default();
        match tz.from_local_datetime(&local) {
            LocalResult::Single(t) => return t.with_timezone(&Utc),
            LocalResult::Ambiguous(first, _) => return first.with_timezone(&Utc),
            LocalResult::None => {}
        }
    }
    date.and_hms_opt(0, 0, 0).unwrap_or_default().and_utc()
}

impl Period {
    /// Checks the dates against the kind and the length limit.
    pub fn new(
        kind: PeriodKind,
        from: NaiveDate,
        to: NaiveDate,
        tz: Tz,
    ) -> Result<Self, PeriodError> {
        if to < from {
            return Err(PeriodError::Reversed);
        }
        let length = (to - from).num_days() + 1;
        if length > MAX_DAYS {
            return Err(PeriodError::TooLong);
        }
        let fits = match kind {
            PeriodKind::Day => length == 1,
            PeriodKind::Week => length == 7,
            PeriodKind::Month => from.day() == 1 && to == last_of_month(from),
            PeriodKind::Custom => true,
        };
        if fits {
            Ok(Self { kind, from, to, tz })
        } else {
            Err(PeriodError::KindMismatch)
        }
    }

    /// The half-open UTC range `[from 00:00, to+1 00:00)` in local time.
    #[must_use]
    pub fn utc_range(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        (
            local_midnight(self.tz, self.from),
            local_midnight(self.tz, self.to + Days::new(1)),
        )
    }

    /// The Previous Period: the same kind, just before this one.
    #[must_use]
    pub fn previous(&self) -> Self {
        let length = (self.to - self.from).num_days() + 1;
        let (from, to) = match self.kind {
            PeriodKind::Month => {
                let to = self.from - Days::new(1);
                (to.with_day(1).unwrap_or(to), to)
            }
            // Day, Week and Custom all step back by their own length.
            _ => (
                self.from - Duration::days(length),
                self.from - Duration::days(1),
            ),
        };
        Self { from, to, ..*self }
    }

    /// Every Day of the Period, in order.
    #[must_use]
    pub fn days(&self) -> Vec<NaiveDate> {
        self.from
            .iter_days()
            .take_while(|d| *d <= self.to)
            .collect()
    }
}
