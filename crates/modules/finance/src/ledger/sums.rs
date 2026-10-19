//! Money in, Money out and Transfers over a set of Transactions. Pure: the rows come from SQL.

use std::collections::BTreeMap;

use chrono::NaiveDate;

use crate::{reader::contract::Direction, transfers};

/// The Transfer a Transaction belongs to, as far as the sums need it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Membership {
    /// The Transfer's id.
    pub transfer_id: i64,
    /// The debit's amount.
    pub debit_paise: i64,
    /// What all its linked credits add up to (also those outside the filter).
    pub credited_paise: i64,
    /// This Transaction is the credit that carries the Transfer's Extra.
    pub carries_extra: bool,
}

/// One Transaction inside the filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Row {
    /// Its date.
    pub date: NaiveDate,
    /// Which way the money moved.
    pub direction: Direction,
    /// Its amount.
    pub amount_paise: i64,
    /// Its Transfer, when it has one.
    pub transfer: Option<Membership>,
}

/// One day's Money in and Money out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Day {
    /// The day.
    pub date: NaiveDate,
    /// Money in.
    pub money_in_paise: i64,
    /// Money out.
    pub money_out_paise: i64,
}

/// The figures of one range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Totals {
    /// Money in.
    pub money_in_paise: i64,
    /// Money out.
    pub money_out_paise: i64,
    /// Moved between the person's own Accounts.
    pub transfers_paise: i64,
    /// Every day of the range, zeros included.
    pub daily: Vec<Day>,
}

/// What the filter sees of one Transfer.
struct Seen {
    moved_paise: i64,
    debit_inside: bool,
    credits_inside_paise: i64,
}

impl Seen {
    /// What this Transfer adds to the Transfers figure: what it moved when its debit is inside
    /// the filter, else what its credits inside received, at most what it moved.
    fn counted(&self) -> i64 {
        if self.debit_inside {
            self.moved_paise
        } else {
            self.credits_inside_paise.min(self.moved_paise)
        }
    }
}

/// An empty day for every date of `from..=to`.
fn empty_days(from: NaiveDate, to: NaiveDate) -> Vec<Day> {
    from.iter_days()
        .take_while(|date| *date <= to)
        .map(|date| Day {
            date,
            money_in_paise: 0,
            money_out_paise: 0,
        })
        .collect()
}

/// Adds one Transaction of a Transfer to its day and to what the filter sees of the Transfer.
fn add_member(day: &mut Day, row: &Row, m: &Membership, seen: &mut BTreeMap<i64, Seen>) {
    let figures = transfers::figures(m.debit_paise, m.credited_paise);
    let entry = seen.entry(m.transfer_id).or_insert(Seen {
        moved_paise: m.debit_paise.min(m.credited_paise),
        debit_inside: false,
        credits_inside_paise: 0,
    });
    match row.direction {
        Direction::Debit => {
            day.money_out_paise += figures.remaining_paise;
            entry.debit_inside = true;
        }
        Direction::Credit => {
            if m.carries_extra {
                day.money_in_paise += figures.extra_paise;
            }
            entry.credits_inside_paise += row.amount_paise;
        }
    }
}

/// Sums `rows` over `from..=to`; rows outside the range are ignored.
///
/// - A Transaction outside any Transfer is Money in (credit) or Money out (debit).
/// - A Transfer is left out of both, except its remainder (Money out, on the debit's date) and
///   its Extra (Money in, on the carrying credit's date).
/// - A Transfer counts once in `transfers_paise` ([`Seen::counted`]), so a filter on one
///   Account still shows it as a Transfer rather than as Money in or out.
#[must_use]
pub fn aggregate(rows: &[Row], from: NaiveDate, to: NaiveDate) -> Totals {
    let mut daily = empty_days(from, to);
    let mut seen = BTreeMap::new();
    for row in rows {
        let index = usize::try_from((row.date - from).num_days()).ok();
        let Some(day) = index.and_then(|i| daily.get_mut(i)) else {
            continue;
        };
        match &row.transfer {
            Some(m) => add_member(day, row, m, &mut seen),
            None => match row.direction {
                Direction::Credit => day.money_in_paise += row.amount_paise,
                Direction::Debit => day.money_out_paise += row.amount_paise,
            },
        }
    }
    Totals {
        money_in_paise: daily.iter().map(|d| d.money_in_paise).sum(),
        money_out_paise: daily.iter().map(|d| d.money_out_paise).sum(),
        transfers_paise: seen.values().map(Seen::counted).sum(),
        daily,
    }
}
