use std::collections::HashMap;

use super::{Evidence, PrintingToAdd, StoredEvidence, read_again_diff};
use crate::{reader::contract::Transaction, test_support::tx};

fn keys(rows: &[Transaction]) -> Vec<StoredEvidence> {
    (1..)
        .zip(rows)
        .map(|(id, row)| StoredEvidence {
            id,
            evidence: Evidence::from(row),
        })
        .collect()
}

/// Every stored row printed here and still found.
fn all_here(count: i64) -> HashMap<i64, bool> {
    (1..=count).map(|id| (id, false)).collect()
}

#[test]
fn read_again_with_no_change_reports_everything_unchanged() {
    let rows = [tx("a", 1), tx("b", 2)];
    let diff = read_again_diff(&rows, &keys(&rows), &all_here(2));
    assert_eq!(diff.unchanged, vec![1, 2]);
    assert!(
        diff.add.is_empty()
            && diff.gone_transaction_ids.is_empty()
            && diff.new_printings.is_empty()
    );
}

#[test]
fn read_again_reports_added_and_no_longer_found() {
    let stored = [tx("a", 1), tx("gone", 2)];
    let incoming = [tx("a", 1), tx("fresh", 3)];
    let diff = read_again_diff(&incoming, &keys(&stored), &all_here(2));
    assert_eq!(
        (diff.add, diff.unchanged, diff.gone_transaction_ids),
        (vec![1], vec![1], vec![2])
    );
}

#[test]
fn read_again_counts_repeated_rows_as_a_multiset() {
    let stored = [tx("x", 1), tx("x", 1), tx("x", 1)];
    let incoming = [tx("x", 1), tx("x", 1)];
    let diff = read_again_diff(&incoming, &keys(&stored), &all_here(3));
    assert_eq!(diff.unchanged, vec![1, 2]);
    assert_eq!(diff.gone_transaction_ids, vec![3]);
    let more = [tx("x", 1), tx("x", 1), tx("x", 1), tx("x", 1)];
    assert_eq!(
        read_again_diff(&more, &keys(&stored), &all_here(3)).add,
        vec![3]
    );
}

#[test]
fn rows_already_gone_are_not_reported_again_but_are_restored_when_found() {
    let rows = [tx("a", 1), tx("b", 2)];
    let here = HashMap::from([(1, true), (2, true)]);
    let diff = read_again_diff(&rows[..1], &keys(&rows), &here);
    assert_eq!(diff.unchanged, vec![1]);
    assert!(diff.gone_transaction_ids.is_empty());
}

#[test]
fn a_row_this_statement_never_printed_is_enriched_not_added() {
    let stored = [tx("a", 1), tx("b", 2)];
    let incoming = [tx("a", 1), tx("b", 2)];
    let here = HashMap::from([(1, false)]);
    let diff = read_again_diff(&incoming, &keys(&stored), &here);
    assert_eq!(
        (diff.unchanged, diff.new_printings),
        (
            vec![1],
            vec![PrintingToAdd {
                index: 1,
                transaction_id: 2
            }]
        )
    );
    assert!(diff.add.is_empty() && diff.gone_transaction_ids.is_empty());
}

#[test]
fn a_twin_this_statement_printed_is_preferred_over_one_it_did_not() {
    let stored = [tx("x", 1), tx("x", 1)];
    let here = HashMap::from([(2, false)]);
    let diff = read_again_diff(&[tx("x", 1)], &keys(&stored), &here);
    assert_eq!(diff.unchanged, vec![2]);
    assert!(diff.new_printings.is_empty() && diff.gone_transaction_ids.is_empty());
}
