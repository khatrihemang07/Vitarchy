//! Rung 2: the debit names the other Account's number.

use super::{AccountInfo, Rung, support::*};

fn accounts() -> Vec<AccountInfo> {
    vec![savings(1, "1111"), loan(2, "7777"), savings(3, "9999")]
}

#[test]
fn a_long_digit_run_ending_in_the_loans_digits_links_the_same_days_credit() {
    let rows = [
        debit(10, 1, 4, 500_000, "DIRECT DR 0000000007777"),
        credit(20, 2, 4, 500_000, "O.S. DEPOSIT TRANSFER"),
    ];
    let found = run(&accounts(), &rows);
    assert_eq!(found.links, vec![link(10, &[20], Rung::AccountNumber)]);
    assert_eq!(found.found.account_number, 1);
}

#[test]
fn same_amount_debits_elsewhere_do_not_confuse_it() {
    let rows = [
        debit(10, 1, 4, 500_000, "DIRECT DR 0000000007777"),
        debit(11, 1, 4, 500_000, "SIP instalment"),
        debit(12, 3, 4, 500_000, "something else"),
        credit(20, 2, 4, 500_000, "O.S. DEPOSIT TRANSFER"),
    ];
    assert_eq!(
        run(&accounts(), &rows).links,
        vec![link(10, &[20], Rung::AccountNumber)]
    );
}

#[test]
fn the_credit_must_be_on_the_named_account() {
    let rows = [
        debit(10, 1, 4, 500_000, "DIRECT DR 0000000007777"),
        credit(20, 3, 4, 500_000, "inward"),
    ];
    assert_eq!(run(&accounts(), &rows).found.account_number, 0);
}

#[test]
fn the_date_and_amount_must_both_agree() {
    let next_day = [
        debit(10, 1, 4, 500_000, "DIRECT DR 0000000007777"),
        credit(20, 2, 5, 500_000, "inward"),
    ];
    assert_eq!(run(&accounts(), &next_day).found.account_number, 0);
    let other_amount = [
        debit(10, 1, 4, 500_000, "DIRECT DR 0000000007777"),
        credit(20, 2, 4, 500_001, "inward"),
    ];
    assert_eq!(run(&accounts(), &other_amount).found.account_number, 0);
}

#[test]
fn runs_shorter_than_eight_digits_are_not_account_numbers() {
    let rows = [
        debit(10, 1, 4, 500, "CHQ 007777"),
        credit(20, 2, 4, 500, "in"),
    ];
    assert_eq!(run(&accounts(), &rows).found.account_number, 0);
}

#[test]
fn any_remembered_digits_of_the_account_count() {
    let mut accounts = accounts();
    accounts[1].digits.push("8888".to_owned());
    let rows = [
        debit(10, 1, 4, 500, "DR 000000008888"),
        credit(20, 2, 4, 500, "in"),
    ];
    assert_eq!(
        run(&accounts, &rows).links,
        vec![link(10, &[20], Rung::AccountNumber)]
    );
}

#[test]
fn digits_shorter_than_four_are_too_weak_to_name_an_account() {
    let mut accounts = accounts();
    accounts[1].digits = vec!["77".to_owned()];
    let rows = [
        debit(10, 1, 4, 500, "DR 000000007777"),
        credit(20, 2, 4, 500, "in"),
    ];
    assert_eq!(run(&accounts, &rows).found.account_number, 0);
}

#[test]
fn a_debit_naming_its_own_account_links_nothing() {
    let rows = [
        debit(10, 2, 4, 500, "DR 000000007777"),
        credit(20, 2, 4, 500, "in"),
    ];
    assert_eq!(run(&accounts(), &rows).found.account_number, 0);
}
