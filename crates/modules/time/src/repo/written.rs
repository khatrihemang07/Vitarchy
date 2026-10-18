//! The outcome of a batched upsert, shared by every table that reports it.

use chrono::{DateTime, Utc};

use crate::timeline::TimeRange;

/// How many rows of a batch were new and how many changed.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Written {
    /// Rows stored for the first time.
    pub inserted: u64,
    /// Existing rows whose fields were written again.
    pub updated: u64,
    /// The stretch of time the written rows cover; `None` when nothing was written.
    pub range: Option<TimeRange>,
}

impl Written {
    /// From the `(inserted, updated)` counts a batch statement returns (`xmax = 0` marks a new row).
    #[must_use]
    pub fn from_counts((inserted, updated): (i64, i64)) -> Self {
        Self {
            inserted: u64::try_from(inserted).unwrap_or(0),
            updated: u64::try_from(updated).unwrap_or(0),
            range: None,
        }
    }

    /// These counts, with the earliest start and latest end of the rows written (both `None`
    /// when none were).
    #[must_use]
    pub fn covering(self, from: Option<DateTime<Utc>>, to: Option<DateTime<Utc>>) -> Self {
        Self {
            range: from.zip(to).map(|(from, to)| TimeRange { from, to }),
            ..self
        }
    }
}

impl Written {
    /// This outcome with the range stretched to also cover `from..to` (the old place of rows
    /// that moved); unchanged when either end is `None`.
    #[must_use]
    pub fn also_covering(self, from: Option<DateTime<Utc>>, to: Option<DateTime<Utc>>) -> Self {
        let Some(extra) = from.zip(to).map(|(from, to)| TimeRange { from, to }) else {
            return self;
        };
        Self {
            range: Some(self.range.map_or(extra, |range| range.union(extra))),
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_span_needs_both_ends_and_nothing_written_has_none() {
        let t = DateTime::from_timestamp(5, 0).unwrap();
        assert_eq!(
            Written::from_counts((1, 0)).covering(Some(t), None).range,
            None
        );
        assert_eq!(Written::default().covering(None, None).range, None);
        let range = Written::from_counts((1, 0))
            .covering(Some(t), Some(t))
            .range;
        assert_eq!(range, Some(TimeRange { from: t, to: t }));
    }

    #[test]
    fn the_old_place_of_a_moved_row_stretches_the_range() {
        let at = |s| DateTime::from_timestamp(s, 0).unwrap();
        let written = Written::from_counts((0, 1)).covering(Some(at(50)), Some(at(60)));
        let stretched = written.also_covering(Some(at(10)), Some(at(55)));
        assert_eq!(
            stretched.range,
            Some(TimeRange {
                from: at(10),
                to: at(60)
            })
        );
        let only_old = Written::default().also_covering(Some(at(1)), Some(at(2)));
        assert_eq!(
            only_old.range,
            Some(TimeRange {
                from: at(1),
                to: at(2)
            })
        );
    }
}
