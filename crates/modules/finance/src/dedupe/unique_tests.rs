//! The last rung: a payment app's row and an issuer's row of one Account that share an
//! amount, a direction and a date within a day, when each has exactly one such partner.

use chrono::Days;

use super::{Evidence, Origin, Origins, StoredEvidence, pair, pair_with};
use crate::{
    reader::contract::{Direction, Transaction},
    test_support::tx,
};

fn on(day_offset: u64, description: &str, amount: i64) -> Transaction {
    let mut row = tx(description, amount);
    row.date = row.date.checked_add_days(Days::new(day_offset)).unwrap();
    row
}

fn stored(origin: Origin, rows: &[Transaction]) -> Vec<StoredEvidence> {
    (1..)
        .zip(rows)
        .map(|(id, row)| {
            StoredEvidence::new(id, Evidence::from(row).with_origins(Origins::of(origin)))
        })
        .collect()
}

#[test]
fn a_lone_payment_app_row_joins_the_lone_issuer_row_of_the_same_amount_and_day() {
    let bank = stored(Origin::Statement, &[on(0, "upi/dr/shop one/hdfc", 12_300)]);
    let app = [on(0, "Paid to SHOP ONE", 12_300)];
    assert_eq!(pair(&app, Origin::PaymentHistory, &bank), vec![Some(1)]);
}

#[test]
fn it_works_the_other_way_round_too() {
    let app = stored(Origin::PaymentHistory, &[on(0, "Paid to SHOP ONE", 12_300)]);
    let bank = [on(1, "upi/dr/shop one/hdfc", 12_300)];
    assert_eq!(pair(&bank, Origin::Statement, &app), vec![Some(1)]);
}

#[test]
fn a_day_either_side_is_allowed_and_two_are_not() {
    let bank = stored(Origin::Statement, &[on(1, "bank text", 500)]);
    assert_eq!(
        pair(&[on(0, "Paid to Cafe", 500)], Origin::PaymentHistory, &bank),
        vec![Some(1)]
    );
    assert_eq!(
        pair(&[on(3, "Paid to Cafe", 500)], Origin::PaymentHistory, &bank),
        vec![None]
    );
}

#[test]
fn a_different_amount_or_direction_never_joins() {
    let bank = stored(Origin::Statement, &[on(0, "upi/dr/grocer/hdfc", 500)]);
    assert_eq!(
        pair(&[on(0, "Paid to Cafe", 501)], Origin::PaymentHistory, &bank),
        vec![None]
    );
    let credit = Transaction {
        direction: Direction::Credit,
        ..on(0, "Paid to Cafe", 500)
    };
    assert_eq!(pair(&[credit], Origin::PaymentHistory, &bank), vec![None]);
}

#[test]
fn two_issuer_rows_in_the_window_refuse_to_merge() {
    let bank = stored(
        Origin::Statement,
        &[
            on(0, "upi/dr/grocer/hdfc", 500),
            on(1, "neft/cr/rent/sbin", 500),
        ],
    );
    assert_eq!(
        pair(&[on(0, "Paid to Cafe", 500)], Origin::PaymentHistory, &bank),
        vec![None]
    );
}

#[test]
fn two_payment_app_rows_in_the_window_refuse_to_merge() {
    let bank = stored(Origin::Statement, &[on(0, "upi/dr/grocer/hdfc", 500)]);
    let app = [on(0, "Paid to Cafe", 500), on(1, "Paid to Tailor", 500)];
    assert_eq!(pair(&app, Origin::PaymentHistory, &bank), vec![None, None]);
}

#[test]
fn two_same_amount_payments_on_both_sides_refuse_to_merge() {
    let bank = stored(
        Origin::Statement,
        &[
            on(0, "upi/dr/grocer/hdfc", 500),
            on(0, "neft/cr/rent/sbin", 500),
        ],
    );
    let app = [on(0, "Paid to Cafe", 500), on(0, "Paid to Tailor", 500)];
    assert_eq!(pair(&app, Origin::PaymentHistory, &bank), vec![None, None]);
}

#[test]
fn a_row_matched_by_stronger_evidence_still_counts_as_a_candidate() {
    // The first app row shares its reference with the bank row. The second app row has the
    // same amount, so it is ambiguous even though the first is spoken for.
    let mut bank_row = on(0, "upi/dr/grocer/hdfc", 500);
    bank_row.reference = Some("R1".to_owned());
    let bank = stored(Origin::Statement, &[bank_row]);
    let mut first = on(0, "Paid to Cafe", 500);
    first.reference = Some("R1".to_owned());
    let second = on(0, "Paid to Tailor", 500);
    assert_eq!(
        pair(&[first, second], Origin::PaymentHistory, &bank),
        vec![Some(1), None]
    );
}

#[test]
fn two_rows_of_the_same_kind_never_join_by_amount_and_date() {
    let bank = stored(Origin::Statement, &[on(0, "upi/dr/grocer/hdfc", 500)]);
    assert_eq!(
        pair(&[on(0, "neft/cr/rent/sbin", 500)], Origin::Statement, &bank),
        vec![None]
    );
    let app = stored(Origin::PaymentHistory, &[on(0, "Paid to Cafe", 500)]);
    assert_eq!(
        pair(
            &[on(0, "Paid to Tailor", 500)],
            Origin::PaymentHistory,
            &app
        ),
        vec![None]
    );
}

#[test]
fn a_transaction_both_kinds_already_printed_is_not_a_candidate() {
    let both = StoredEvidence::new(
        1,
        Evidence::from(&on(0, "upi/dr/grocer/hdfc", 500)).with_origins(Origins::Both),
    );
    assert_eq!(
        pair(
            &[on(0, "Paid to Cafe", 500)],
            Origin::PaymentHistory,
            &[both]
        ),
        vec![None]
    );
}

#[test]
fn rows_with_no_known_origin_are_left_to_the_other_rungs() {
    let rows = [on(0, "one text", 500)];
    let unknown = vec![StoredEvidence::new(
        1,
        Evidence::from(&on(0, "another", 500)),
    )];
    let incoming: Vec<Evidence> = rows.iter().map(Evidence::from).collect();
    assert_eq!(pair_with(&incoming, &unknown), vec![None]);
}

#[test]
fn stronger_rungs_still_win_over_the_unique_rung() {
    let mut bank_row = on(0, "upi/dr/grocer/hdfc", 500);
    bank_row.balance_paise = Some(70);
    let other = on(0, "other bank text", 500);
    let bank = stored(Origin::Statement, &[other, bank_row]);
    let mut app_row = on(0, "Paid to Cafe", 500);
    app_row.reference = Some("R9".to_owned());
    app_row.source_id = Some("T9".to_owned());
    // Two bank rows share the window, so the unique rung refuses; nothing else matches.
    assert_eq!(pair(&[app_row], Origin::PaymentHistory, &bank), vec![None]);
}

#[test]
fn rows_whose_references_disagree_never_join_by_amount_and_date() {
    let mut bank_row = on(0, "upi/dr/grocer/hdfc", 500);
    bank_row.reference = Some("R1".to_owned());
    let bank = stored(Origin::Statement, &[bank_row]);
    let mut app_row = on(0, "Paid to Cafe", 500);
    app_row.reference = Some("R2".to_owned());
    assert_eq!(
        pair(&[app_row.clone()], Origin::PaymentHistory, &bank),
        vec![None]
    );
    app_row.reference = None;
    assert_eq!(
        pair(&[app_row], Origin::PaymentHistory, &bank),
        vec![Some(1)]
    );
}
