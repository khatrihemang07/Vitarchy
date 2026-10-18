use chrono::Days;

use super::{Evidence, Origin, Origins, StoredEvidence, new_rows, pair, pairing::pair_evidence};
use crate::{reader::contract::Transaction, test_support::tx};

fn stored_of(rows: &[Transaction]) -> Vec<Evidence> {
    rows.iter().map(Evidence::from).collect()
}

fn keys(rows: &[Transaction]) -> Vec<StoredEvidence> {
    (1..)
        .zip(stored_of(rows))
        .map(|(id, evidence)| StoredEvidence::new(id, evidence))
        .collect()
}

fn balanced(description: &str, amount: i64, balance: i64) -> Transaction {
    let mut row = tx(description, amount);
    row.balance_paise = Some(balance);
    row
}

#[test]
fn two_identical_rows_with_one_stored_adds_one() {
    let rows = [tx("coffee shop visit", 100), tx("coffee shop visit", 100)];
    assert_eq!(new_rows(&rows, &stored_of(&rows[..1])), vec![1]);
}

#[test]
fn a_file_repeating_stored_rows_adds_nothing() {
    let rows = [tx("a", 1), tx("b", 2), tx("a", 1)];
    assert!(new_rows(&rows, &stored_of(&rows)).is_empty());
}

#[test]
fn nothing_stored_means_everything_is_new_in_order() {
    let rows = [tx("a", 1), tx("b", 2)];
    assert_eq!(new_rows(&rows, &[]), vec![0, 1]);
}

#[test]
fn a_truncated_narration_is_the_same_row_when_the_balance_agrees() {
    let stored = [balanced(
        "neft cr acme corp salary for june 2026",
        5000,
        9000,
    )];
    let incoming = [balanced("neft cr acme", 5000, 9000)];
    assert!(new_rows(&incoming, &stored_of(&stored)).is_empty());
}

#[test]
fn same_date_and_amount_with_different_balances_are_two_rows() {
    let stored = [balanced("atm cash withdrawal", 2000, 8000)];
    let incoming = [balanced("atm cash withdrawal", 2000, 6000)];
    assert_eq!(new_rows(&incoming, &stored_of(&stored)), vec![0]);
}

#[test]
fn a_day_off_matches_with_a_balance_but_not_without() {
    let mut shifted = balanced("coffee", 100, 50);
    shifted.date = shifted.date.checked_add_days(Days::new(1)).unwrap();
    assert!(
        new_rows(
            &[shifted.clone()],
            &stored_of(&[balanced("coffee", 100, 50)])
        )
        .is_empty()
    );
    shifted.balance_paise = None;
    assert_eq!(
        new_rows(&[shifted], &stored_of(&[tx("coffee shop visit", 100)])),
        vec![0]
    );
}

#[test]
fn strong_evidence_wins_over_a_weaker_match_that_came_first() {
    // The first incoming row would match the stored row by description, but the second
    // shares its balance; the balance pairing must take the stored row.
    let stored = [balanced("coffee shop visit", 100, 70)];
    let incoming = [
        balanced("coffee shop visit", 100, 99),
        balanced("other", 100, 70),
    ];
    assert_eq!(
        pair_evidence(&stored_of(&incoming), &stored_of(&stored)),
        vec![None, Some(0)]
    );
}

#[test]
fn pairing_finds_the_same_number_of_pairs_when_the_roles_swap() {
    let a = [
        balanced("neft cr acme corp salary", 5000, 1),
        tx("upi shop one", 100),
        tx("upi shop one", 100),
        tx("rent payment", 700),
    ];
    let b = [
        balanced("neft cr acme", 5000, 1),
        tx("upi shop one", 100),
        tx("rent payment march", 700),
    ];
    let count = |x: &[Transaction], y: &[Transaction]| {
        pair_evidence(&stored_of(x), &stored_of(y))
            .iter()
            .flatten()
            .count()
    };
    assert_eq!(count(&a, &b), count(&b, &a));
    assert_eq!(count(&a, &b), 3);
}

#[test]
fn pair_names_the_stored_id_each_incoming_row_is() {
    let stored = keys(&[tx("a", 1), tx("b", 2)]);
    let incoming = [tx("b", 2), tx("fresh", 3), tx("a", 1)];
    assert_eq!(
        pair(&incoming, Origin::Statement, &stored),
        vec![Some(2), None, Some(1)]
    );
}

#[test]
fn new_rows_are_exactly_the_rows_pair_leaves_unmatched() {
    let stored_rows = [balanced("coffee shop visit", 100, 70), tx("rent", 700)];
    let incoming = [
        balanced("coffee shop", 100, 70),
        tx("rent", 700),
        tx("rent", 700),
        tx("new", 5),
    ];
    let unmatched: Vec<usize> = pair(&incoming, Origin::Statement, &keys(&stored_rows))
        .iter()
        .enumerate()
        .filter(|(_, id)| id.is_none())
        .map(|(index, _)| index)
        .collect();
    assert_eq!(new_rows(&incoming, &stored_of(&stored_rows)), unmatched);
    assert_eq!(unmatched, vec![2, 3]);
}

fn with_id(mut row: Transaction, id: &str) -> Transaction {
    row.source_id = Some(id.to_owned());
    row
}

fn printed_by(row: &Transaction, origin: Origin) -> Evidence {
    Evidence::from(row).with_origins(Origins::of(origin))
}

#[test]
fn a_row_is_the_same_as_a_transaction_when_it_matches_any_of_its_printings() {
    // The shown view carries the bank's id; the payment app's own id lives in its Printing.
    let shown = with_id(tx("upi shop one", 100), "BANK1");
    let app_printing = with_id(tx("paid to shop one", 100), "APP1");
    let mut row = StoredEvidence::new(7, Evidence::from(&shown).with_origins(Origins::Both));
    row.printed = vec![
        printed_by(&shown, Origin::Statement),
        printed_by(&app_printing, Origin::PaymentHistory),
    ];

    let again = with_id(tx("paid to shop one", 100), "APP1");

    assert_eq!(
        pair(&[again], Origin::PaymentHistory, &[row]),
        vec![Some(7)]
    );
}

#[test]
fn a_disagreeing_id_of_the_same_kind_still_blocks_when_no_printing_agrees() {
    let shown = with_id(tx("paid to shop one", 100), "APP1");
    let mut row = StoredEvidence::new(7, printed_by(&shown, Origin::PaymentHistory));
    row.printed = vec![printed_by(&shown, Origin::PaymentHistory)];

    let other = with_id(tx("paid to shop one", 100), "APP9");

    assert_eq!(pair(&[other], Origin::PaymentHistory, &[row]), vec![None]);
}
