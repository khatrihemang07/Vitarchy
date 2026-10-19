//! The ladder as a whole: strongest rung first, nothing used twice, nothing outside the
//! focus, nothing the person unlinked.

use std::collections::HashSet;

use super::{Detection, Input, Rung, detect, support::*};

fn accounts() -> Vec<super::AccountInfo> {
    vec![savings(1, "1111"), savings(2, "2222"), savings(3, "3333")]
}

#[test]
fn a_stronger_rung_takes_a_pair_before_a_weaker_one() {
    let rows = [
        referenced(debit(10, 1, 5, 50_000, "out"), "REF123456789"),
        credit(20, 2, 5, 50_000, "in REF123456789"),
    ];
    let found = run(&accounts(), &rows);
    assert_eq!(found.links, vec![link(10, &[20], Rung::Reference)]);
    assert_eq!(found.found.unique_amount, 0);
}

#[test]
fn a_transaction_used_by_one_rung_is_not_used_again() {
    // Credit 20 satisfies rung 1 for debit 10; debit 11 (same amount) must not also take it.
    let rows = [
        referenced(debit(10, 1, 5, 50_000, "out"), "REF123456789"),
        debit(11, 3, 5, 50_000, "unrelated"),
        credit(20, 2, 5, 50_000, "in REF123456789"),
    ];
    let found = run(&accounts(), &rows);
    assert_eq!(found.links, vec![link(10, &[20], Rung::Reference)]);
}

#[test]
fn stored_transfers_keep_their_transactions() {
    let rows = [debit(10, 1, 5, 500, "out"), credit(20, 2, 5, 500, "in")];
    let stored = [stored(10, &[20], 500, Rung::Manual)];
    assert!(
        run_with(&accounts(), &rows, &stored, &HashSet::new())
            .links
            .is_empty()
    );
}

#[test]
fn a_stored_transfer_still_counts_as_a_rival_for_the_unique_amount_rung() {
    let rows = [
        debit(10, 1, 5, 500, "out"),
        credit(20, 2, 5, 500, "in"),
        debit(11, 3, 5, 500, "out too"),
    ];
    let stored = [stored(10, &[20], 500, Rung::Manual)];
    assert!(
        run_with(&accounts(), &rows, &stored, &HashSet::new())
            .links
            .is_empty()
    );
}

#[test]
fn rejected_pairs_are_not_linked_on_any_rung() {
    let rows = [
        referenced(debit(10, 1, 5, 500, "out"), "REF123456789"),
        credit(20, 2, 5, 500, "in REF123456789"),
    ];
    let rejected: HashSet<(i64, i64)> = [(10, 20)].into_iter().collect();
    assert_eq!(
        run_with(&accounts(), &rows, &[], &rejected),
        Detection::default()
    );
}

#[test]
fn only_transfers_whose_debit_is_in_the_focus_are_reported() {
    let rows = [
        debit(10, 1, 5, 500, "out"),
        credit(20, 2, 5, 500, "in"),
        debit(11, 1, 25, 900, "out"),
        credit(21, 2, 25, 900, "in"),
    ];
    let rejected = HashSet::new();
    let found = detect(&Input {
        accounts: &accounts(),
        candidates: &rows,
        transfers: &[],
        rejected: &rejected,
        focus: Some((day(1), day(10))),
    });
    assert_eq!(found.links, vec![link(10, &[20], Rung::UniqueAmount)]);
    assert_eq!(found.found.unique_amount, 1);
}

#[test]
fn an_empty_day_finds_nothing() {
    assert_eq!(run(&accounts(), &[]), Detection::default());
}
