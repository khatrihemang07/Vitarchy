//! Rung 4: the only debit and the only credit of an amount around that day.

use super::{Rung, support::*};

fn accounts() -> Vec<super::AccountInfo> {
    vec![savings(1, "1111"), savings(2, "2222"), savings(3, "3333")]
}

#[test]
fn a_lone_debit_and_credit_of_one_amount_link_within_a_day() {
    let rows = [
        debit(10, 1, 5, 123_400, "out"),
        credit(20, 2, 6, 123_400, "in"),
    ];
    let found = run(&accounts(), &rows);
    assert_eq!(found.links, vec![link(10, &[20], Rung::UniqueAmount)]);
    assert_eq!(found.found.unique_amount, 1);
}

#[test]
fn two_days_apart_is_too_far() {
    let rows = [
        debit(10, 1, 5, 123_400, "out"),
        credit(20, 2, 7, 123_400, "in"),
    ];
    assert!(run(&accounts(), &rows).links.is_empty());
}

#[test]
fn a_second_debit_of_that_amount_makes_it_ambiguous() {
    let rows = [
        debit(10, 1, 5, 500, "out"),
        debit(11, 3, 5, 500, "other out"),
        credit(20, 2, 5, 500, "in"),
    ];
    let found = run(&accounts(), &rows);
    assert!(found.links.is_empty());
    assert_eq!(found.ambiguous, 2);
}

#[test]
fn a_second_credit_of_that_amount_makes_it_ambiguous() {
    let rows = [
        debit(10, 1, 5, 500, "out"),
        credit(20, 2, 5, 500, "in"),
        credit(21, 3, 6, 500, "in too"),
    ];
    assert!(run(&accounts(), &rows).links.is_empty());
}

#[test]
fn a_same_amount_debit_on_the_receiving_account_is_no_rival() {
    let rows = [
        debit(10, 1, 5, 500, "out"),
        credit(20, 2, 5, 500, "in"),
        debit(11, 2, 5, 500, "out of the receiving account"),
    ];
    // Debit 11 could never pair with credit 20 (one Account), so it does not rival debit 10.
    assert_eq!(
        run(&accounts(), &rows).links,
        vec![link(10, &[20], Rung::UniqueAmount)]
    );
}

#[test]
fn rows_far_in_time_are_not_rivals() {
    let rows = [
        debit(10, 1, 5, 500, "out"),
        debit(11, 3, 20, 500, "months later"),
        credit(20, 2, 5, 500, "in"),
    ];
    assert_eq!(
        run(&accounts(), &rows).links,
        vec![link(10, &[20], Rung::UniqueAmount)]
    );
}

#[test]
fn the_same_account_never_pairs_with_itself() {
    let rows = [debit(10, 1, 5, 500, "out"), credit(20, 1, 5, 500, "in")];
    assert!(run(&accounts(), &rows).links.is_empty());
}
