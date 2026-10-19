use chrono::NaiveDate;

use super::{Membership, Row, Totals, aggregate};
use crate::reader::contract::Direction::{Credit, Debit};

fn d(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 7, day).unwrap()
}

fn plain(day: u32, direction: crate::reader::contract::Direction, amount_paise: i64) -> Row {
    Row {
        date: d(day),
        direction,
        amount_paise,
        transfer: None,
    }
}

fn member(
    day: u32,
    direction: crate::reader::contract::Direction,
    amount_paise: i64,
    (debit_paise, credited_paise): (i64, i64),
    carries_extra: bool,
) -> Row {
    Row {
        transfer: Some(Membership {
            transfer_id: 1,
            debit_paise,
            credited_paise,
            carries_extra,
        }),
        ..plain(day, direction, amount_paise)
    }
}

fn run(rows: &[Row]) -> Totals {
    aggregate(rows, d(1), d(3))
}

#[test]
fn plain_credits_and_debits_are_money_in_and_out() {
    let t = run(&[
        plain(1, Credit, 500),
        plain(2, Debit, 200),
        plain(2, Debit, 50),
    ]);
    assert_eq!(
        (t.money_in_paise, t.money_out_paise, t.transfers_paise),
        (500, 250, 0)
    );
}

#[test]
fn every_day_of_the_range_is_listed_with_zeros() {
    let t = run(&[plain(2, Debit, 200)]);
    let days: Vec<_> = t
        .daily
        .iter()
        .map(|x| (x.date, x.money_in_paise, x.money_out_paise))
        .collect();
    assert_eq!(days, [(d(1), 0, 0), (d(2), 0, 200), (d(3), 0, 0)]);
}

#[test]
fn a_whole_transfer_is_neither_in_nor_out() {
    let t = run(&[
        member(1, Debit, 1_000, (1_000, 1_000), false),
        member(2, Credit, 1_000, (1_000, 1_000), false),
    ]);
    assert_eq!(
        (t.money_in_paise, t.money_out_paise, t.transfers_paise),
        (0, 0, 1_000)
    );
}

#[test]
fn a_partial_transfers_remainder_is_money_out_on_the_debit_date() {
    let t = run(&[
        member(1, Debit, 1_000, (1_000, 400), false),
        member(3, Credit, 400, (1_000, 400), false),
    ]);
    assert_eq!(
        (t.money_in_paise, t.money_out_paise, t.transfers_paise),
        (0, 600, 400)
    );
    assert_eq!(t.daily[0].money_out_paise, 600);
}

#[test]
fn an_extra_is_money_in_on_the_carrying_credits_date() {
    let t = run(&[
        member(1, Debit, 1_000, (1_000, 1_030), false),
        member(2, Credit, 600, (1_000, 1_030), false),
        member(3, Credit, 430, (1_000, 1_030), true),
    ]);
    assert_eq!(
        (t.money_in_paise, t.money_out_paise, t.transfers_paise),
        (30, 0, 1_000)
    );
    assert_eq!(t.daily[2].money_in_paise, 30);
}

#[test]
fn one_side_alone_still_counts_as_a_transfer_not_in_or_out() {
    let debit_only = run(&[member(1, Debit, 1_000, (1_000, 1_000), false)]);
    let credit_only = run(&[member(2, Credit, 1_000, (1_000, 1_000), false)]);
    assert_eq!(
        (debit_only.money_out_paise, debit_only.transfers_paise),
        (0, 1_000)
    );
    assert_eq!(
        (credit_only.money_in_paise, credit_only.transfers_paise),
        (0, 1_000)
    );
}

#[test]
fn credits_in_the_filter_alone_count_no_more_than_was_moved() {
    let t = run(&[member(2, Credit, 400, (1_000, 400), false)]);
    assert_eq!(t.transfers_paise, 400);
}

#[test]
fn rows_outside_the_range_are_ignored() {
    let t = aggregate(&[plain(9, Credit, 500)], d(1), d(3));
    assert_eq!(t.money_in_paise, 0);
}
