use chrono::Days;

use super::{Evidence, StoredEvidence, new_rows, pair, read_again_diff};
use crate::{reader::contract::Transaction, test_support::tx};

fn stored_of(rows: &[Transaction]) -> Vec<Evidence> {
    rows.iter().map(Evidence::from).collect()
}

fn keys(rows: &[Transaction]) -> Vec<StoredEvidence> {
    (1..)
        .zip(stored_of(rows))
        .map(|(id, evidence)| StoredEvidence {
            id,
            evidence,
            lost: false,
        })
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
        pair(&stored_of(&incoming), &stored_of(&stored)),
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
        pair(&stored_of(x), &stored_of(y)).iter().flatten().count()
    };
    assert_eq!(count(&a, &b), count(&b, &a));
    assert_eq!(count(&a, &b), 3);
}

#[test]
fn read_again_with_no_change_reports_everything_unchanged() {
    let rows = [tx("a", 1), tx("b", 2)];
    let diff = read_again_diff(&rows, &keys(&rows));
    assert_eq!(diff.unchanged, vec![1, 2]);
    assert!(diff.add.is_empty() && diff.no_longer_found.is_empty());
}

#[test]
fn read_again_reports_added_and_no_longer_found() {
    let stored = [tx("a", 1), tx("gone", 2)];
    let incoming = [tx("a", 1), tx("fresh", 3)];
    let diff = read_again_diff(&incoming, &keys(&stored));
    assert_eq!(
        (diff.add, diff.unchanged, diff.no_longer_found),
        (vec![1], vec![1], vec![2])
    );
}

#[test]
fn read_again_counts_duplicates_as_a_multiset() {
    let stored = [tx("x", 1), tx("x", 1), tx("x", 1)];
    let incoming = [tx("x", 1), tx("x", 1)];
    let diff = read_again_diff(&incoming, &keys(&stored));
    assert_eq!(diff.unchanged, vec![1, 2]);
    assert_eq!(diff.no_longer_found, vec![3]);
    let more = [tx("x", 1), tx("x", 1), tx("x", 1), tx("x", 1)];
    assert_eq!(read_again_diff(&more, &keys(&stored)).add, vec![3]);
}

#[test]
fn rows_already_marked_lost_are_not_reported_again_but_are_restored_when_found() {
    let rows = [tx("a", 1), tx("b", 2)];
    let mut stored = keys(&rows);
    stored[0].lost = true;
    stored[1].lost = true;
    let diff = read_again_diff(&rows[..1], &stored);
    assert_eq!(diff.unchanged, vec![1]);
    assert!(diff.no_longer_found.is_empty());
}
