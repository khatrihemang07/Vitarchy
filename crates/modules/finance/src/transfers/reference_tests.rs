//! Rung 1: the debit's reference appears in the credit.

use super::{Rung, support::*};

const UTR: &str = "AX12345678";

fn accounts() -> Vec<super::AccountInfo> {
    vec![savings(1, "1111"), savings(2, "2222")]
}

fn sender() -> super::Candidate {
    referenced(debit(10, 1, 5, 4_000_000, "NEFT out to self"), UTR)
}

#[test]
fn a_reference_inside_the_credits_description_links_them() {
    let rows = [
        sender(),
        credit(20, 2, 5, 4_000_000, &format!("NEFT-{UTR}-JOHN DOE")),
    ];
    let found = run(&accounts(), &rows);
    assert_eq!(found.links, vec![link(10, &[20], Rung::Reference)]);
    assert_eq!(found.found.reference, 1);
}

#[test]
fn any_printing_of_the_credit_may_hold_the_reference() {
    let rows = [
        sender(),
        also_printed(
            credit(20, 2, 5, 4_000_000, "short text"),
            &format!("long text {UTR} more"),
        ),
    ];
    assert_eq!(
        run(&accounts(), &rows).links,
        vec![link(10, &[20], Rung::Reference)]
    );
}

#[test]
fn the_credits_own_reference_may_be_the_one() {
    let rows = [
        sender(),
        referenced(credit(20, 2, 6, 4_000_000, "inward"), "ax12345678"),
    ];
    assert_eq!(
        run(&accounts(), &rows).links,
        vec![link(10, &[20], Rung::Reference)]
    );
}

#[test]
fn a_reference_only_written_in_the_debits_description_is_not_evidence() {
    let rows = [
        also_printed(debit(10, 1, 5, 500, "NEFT out"), "NEFT out ref ZZ99887766"),
        credit(20, 2, 5, 500, "inward ZZ99887766"),
    ];
    assert_eq!(run(&accounts(), &rows).found.reference, 0);
}

#[test]
fn a_day_apart_is_allowed_and_two_are_not() {
    let near = [sender(), credit(20, 2, 6, 4_000_000, &format!("x {UTR}"))];
    assert_eq!(run(&accounts(), &near).links.len(), 1);
    let far = [sender(), credit(20, 2, 7, 4_000_000, &format!("x {UTR}"))];
    assert_eq!(run(&accounts(), &far).found.reference, 0);
}

#[test]
fn amounts_must_be_exact() {
    let rows = [sender(), credit(20, 2, 5, 4_000_001, &format!("x {UTR}"))];
    assert_eq!(run(&accounts(), &rows).found.reference, 0);
}

#[test]
fn short_references_are_not_evidence() {
    let rows = [
        referenced(debit(10, 1, 5, 500, "out"), "AB12345"),
        credit(20, 2, 5, 500, "in AB12345"),
    ];
    assert_eq!(run(&accounts(), &rows).found.reference, 0);
}

#[test]
fn the_reference_must_be_a_whole_token() {
    let rows = [
        sender(),
        credit(20, 2, 5, 4_000_000, &format!("x {UTR}99 y")),
    ];
    assert_eq!(run(&accounts(), &rows).found.reference, 0);
}

#[test]
fn both_ends_must_be_on_different_accounts() {
    let rows = [sender(), credit(20, 1, 5, 4_000_000, &format!("x {UTR}"))];
    assert_eq!(run(&accounts(), &rows).found.reference, 0);
}

#[test]
fn two_credits_naming_the_reference_are_left_ambiguous() {
    let accounts = [savings(1, "1111"), savings(2, "2222"), savings(3, "3333")];
    let rows = [
        sender(),
        credit(20, 2, 5, 4_000_000, &format!("x {UTR}")),
        credit(21, 3, 5, 4_000_000, &format!("y {UTR}")),
    ];
    let found = run(&accounts, &rows);
    assert_eq!(found.found.reference, 0);
    assert!(found.ambiguous >= 1);
}

#[test]
fn a_credit_on_an_unlisted_account_is_ignored() {
    let rows = [sender(), credit(20, 9, 5, 4_000_000, &format!("x {UTR}"))];
    assert_eq!(run(&accounts(), &rows).found.reference, 0);
}
