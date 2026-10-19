//! Rung 3: a card-bill payment and the receipts it paid.

use std::collections::HashSet;

use super::{AccountInfo, Rung, support::*};

const PAYEE: &str = "UPI/pay@cred.club/CRED Club bill";
const RECEIPT: &str = "BBPS Payment received";

fn accounts() -> Vec<AccountInfo> {
    vec![
        savings(1, "1111"),
        card(2, "2222"),
        card(3, "3333"),
        card(4, "4444"),
    ]
}

#[test]
fn a_payment_and_its_exact_receipt_on_the_same_day_link() {
    let rows = [
        debit(10, 1, 5, 1_250_000, PAYEE),
        credit(20, 2, 5, 1_250_000, RECEIPT),
    ];
    let found = run(&accounts(), &rows);
    assert_eq!(found.links, vec![link(10, &[20], Rung::BillPayment)]);
    assert_eq!(found.found.bill_payment, 1);
}

#[test]
fn the_receipt_may_be_a_day_either_side() {
    for receipt_day in [4, 6] {
        let rows = [
            debit(10, 1, 5, 1_000, PAYEE),
            credit(20, 2, receipt_day, 1_000, "PAYMENT RECEIVED 889900"),
        ];
        assert_eq!(run(&accounts(), &rows).links.len(), 1, "day {receipt_day}");
    }
    let rows = [
        debit(10, 1, 5, 1_000, PAYEE),
        credit(20, 2, 7, 1_000, RECEIPT),
    ];
    assert!(run(&accounts(), &rows).links.is_empty());
}

#[test]
fn a_receipt_a_little_over_the_payment_still_links_and_the_extra_is_allowed() {
    let rows = [
        debit(10, 1, 5, 100_000, PAYEE),
        credit(20, 2, 5, 100_000 + 500, "BBPS-PAYMENT"),
    ];
    assert_eq!(
        run(&accounts(), &rows).links,
        vec![link(10, &[20], Rung::BillPayment)]
    );
}

#[test]
fn more_than_fifty_rupees_over_is_not_a_bill_payment() {
    let rows = [
        debit(10, 1, 5, 100_000, PAYEE),
        credit(20, 2, 5, 100_000 + 5_001, RECEIPT),
    ];
    assert!(run(&accounts(), &rows).links.is_empty());
}

#[test]
fn a_payment_split_across_two_cards_links_both_receipts() {
    let rows = [
        debit(10, 1, 5, 2_881_183, PAYEE),
        credit(20, 2, 5, 574_700, RECEIPT),
        credit(21, 3, 5, 2_306_483, "PAYMENT RECEIVED 1"),
    ];
    assert_eq!(
        run(&accounts(), &rows).links,
        vec![link(10, &[20, 21], Rung::BillPayment)]
    );
}

#[test]
fn a_split_may_run_to_four_receipts() {
    let four = [
        debit(10, 1, 5, 4_000, PAYEE),
        credit(20, 2, 5, 1_000, RECEIPT),
        credit(21, 3, 5, 1_000, RECEIPT),
        credit(22, 4, 5, 1_000, RECEIPT),
        credit(23, 4, 5, 1_000, RECEIPT),
    ];
    assert_eq!(
        run(&accounts(), &four).links,
        vec![link(10, &[20, 21, 22, 23], Rung::BillPayment)]
    );
}

#[test]
fn an_exact_match_is_preferred_to_one_with_extra() {
    let rows = [
        debit(10, 1, 5, 100_000, PAYEE),
        credit(20, 2, 5, 100_000, RECEIPT),
        credit(21, 3, 5, 100_300, RECEIPT),
    ];
    assert_eq!(
        run(&accounts(), &rows).links,
        vec![link(10, &[20], Rung::BillPayment)]
    );
}

#[test]
fn two_equally_good_receipts_leave_the_payment_ambiguous() {
    let rows = [
        debit(10, 1, 5, 100_000, PAYEE),
        credit(20, 2, 5, 100_000, RECEIPT),
        credit(21, 3, 5, 100_000, RECEIPT),
    ];
    let found = run(&accounts(), &rows);
    assert!(found.links.is_empty(), "{:?}", found.links);
    assert!(found.ambiguous >= 1);
}

#[test]
fn one_payment_per_card_on_one_day_each_find_their_own_receipt() {
    let rows = [
        debit(10, 1, 5, 100_000, PAYEE),
        debit(11, 1, 5, 250_000, PAYEE),
        credit(20, 2, 5, 100_000, RECEIPT),
        credit(21, 3, 5, 250_400, RECEIPT),
    ];
    let found = run(&accounts(), &rows);
    assert_eq!(
        found.links,
        vec![
            link(10, &[20], Rung::BillPayment),
            link(11, &[21], Rung::BillPayment)
        ]
    );
}

#[test]
fn two_payments_of_one_amount_with_two_such_receipts_stay_ambiguous() {
    let rows = [
        debit(10, 1, 5, 100_000, PAYEE),
        debit(11, 1, 5, 100_000, PAYEE),
        credit(20, 2, 5, 100_000, RECEIPT),
        credit(21, 3, 5, 100_000, RECEIPT),
    ];
    assert!(run(&accounts(), &rows).links.is_empty());
}

#[test]
fn the_payee_marker_is_needed_on_the_debit_and_the_receipt_marker_on_the_card_credit() {
    let no_payee = [
        debit(10, 1, 5, 1_000, "UPI/grocer"),
        credit(20, 2, 5, 1_000, RECEIPT),
    ];
    assert_eq!(run(&accounts(), &no_payee).found.bill_payment, 0);
    let no_receipt = [
        debit(10, 1, 5, 1_000, PAYEE),
        credit(20, 2, 5, 1_000, "refund from shop"),
    ];
    assert_eq!(run(&accounts(), &no_receipt).found.bill_payment, 0);
    let not_a_card = [
        debit(10, 1, 5, 1_000, PAYEE),
        credit(20, 4, 5, 1_000, RECEIPT),
    ];
    let mut accounts = accounts();
    accounts[3] = savings(4, "4444");
    assert_eq!(run(&accounts, &not_a_card).found.bill_payment, 0);
}

#[test]
fn a_marker_in_any_printing_counts() {
    let rows = [
        also_printed(debit(10, 1, 5, 1_000, "UPI/xyz"), "paid via cred.club"),
        also_printed(credit(20, 2, 5, 1_000, "CR"), "BBPS 1234"),
    ];
    assert_eq!(run(&accounts(), &rows).links.len(), 1);
}

#[test]
fn a_lone_receipt_for_a_bigger_payment_makes_a_partial_transfer() {
    let rows = [
        debit(10, 1, 5, 2_881_183, PAYEE),
        credit(20, 2, 5, 574_700, RECEIPT),
    ];
    assert_eq!(
        run(&accounts(), &rows).links,
        vec![link(10, &[20], Rung::BillPayment)]
    );
}

#[test]
fn a_partial_receipt_needs_exactly_one_payment_with_room() {
    let rows = [
        debit(10, 1, 5, 2_000_000, PAYEE),
        debit(11, 1, 5, 3_000_000, PAYEE),
        credit(20, 2, 5, 574_700, RECEIPT),
    ];
    let found = run(&accounts(), &rows);
    assert!(found.links.is_empty());
    assert_eq!(found.ambiguous, 1);
}

#[test]
fn a_receipt_larger_than_the_payment_allows_does_not_fit_even_partially() {
    let rows = [
        debit(10, 1, 5, 1_000, PAYEE),
        credit(20, 2, 5, 9_000, RECEIPT),
    ];
    assert!(run(&accounts(), &rows).links.is_empty());
}

#[test]
fn a_later_receipt_completes_a_partial_transfer() {
    let rows = [
        debit(10, 1, 5, 2_881_183, PAYEE),
        credit(20, 2, 5, 574_700, RECEIPT),
        credit(21, 3, 6, 2_306_483, RECEIPT),
    ];
    let stored = [stored(10, &[20], 574_700, Rung::BillPayment)];
    let found = run_with(&accounts(), &rows, &stored, &HashSet::default());
    assert_eq!(found.links, vec![link(10, &[21], Rung::BillPayment)]);
}

#[test]
fn a_completed_payment_takes_no_more_receipts() {
    let rows = [
        debit(10, 1, 5, 100_000, PAYEE),
        credit(20, 2, 5, 100_000, RECEIPT),
        credit(21, 3, 5, 400, RECEIPT),
    ];
    let stored = [stored(10, &[20], 100_000, Rung::BillPayment)];
    assert!(
        run_with(&accounts(), &rows, &stored, &HashSet::default())
            .links
            .is_empty()
    );
}

#[test]
fn receipts_that_together_overfill_the_one_payment_with_room_are_left_alone() {
    let rows = [
        debit(10, 1, 5, 100_000, PAYEE),
        credit(20, 2, 5, 99_000, RECEIPT),
        credit(21, 3, 5, 99_000, RECEIPT),
    ];
    let found = run(&accounts(), &rows);
    assert!(found.links.is_empty());
    assert_eq!(found.ambiguous, 2);
}

#[test]
fn a_payment_from_the_cards_own_account_is_not_a_transfer() {
    let rows = [
        debit(10, 2, 5, 1_000, PAYEE),
        credit(20, 2, 5, 1_000, RECEIPT),
    ];
    assert!(run(&accounts(), &rows).links.is_empty());
}

#[test]
fn an_unlinked_pair_is_never_linked_again() {
    let rows = [
        debit(10, 1, 5, 1_000, PAYEE),
        credit(20, 2, 5, 1_000, RECEIPT),
    ];
    let rejected = [(10, 20)].into_iter().collect();
    assert!(
        run_with(&accounts(), &rows, &[], &rejected)
            .links
            .is_empty()
    );
}
