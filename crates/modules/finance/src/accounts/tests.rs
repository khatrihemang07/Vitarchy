use super::{AccountIdentity, Match, find, last4, mask::visible_digits, same_number};
use crate::reader::contract::AccountKind::{CreditCard, Loan, Savings};

fn identity(
    account_id: i64,
    issuer: &str,
    kind: crate::reader::contract::AccountKind,
    mask: &str,
) -> AccountIdentity {
    AccountIdentity {
        account_id,
        issuer: issuer.to_owned(),
        kind,
        mask: Some(mask.to_owned()),
    }
}

#[test]
fn last4_reads_the_trailing_digits_of_each_mask_style() {
    assert_eq!(last4(Some("XXXXXX4321")).as_deref(), Some("4321"));
    assert_eq!(last4(Some("4123XXXXXXXX2003")).as_deref(), Some("2003"));
    assert_eq!(last4(Some("XX9971")).as_deref(), Some("9971"));
    assert_eq!(last4(Some("00000012345678")).as_deref(), Some("5678"));
}

#[test]
fn last4_is_none_when_the_digits_are_not_visible() {
    assert_eq!(last4(None), None);
    assert_eq!(last4(Some("")), None);
    assert_eq!(last4(Some("1234XXXX")), None);
    assert_eq!(last4(Some("XXX123")), None);
}

#[test]
fn matches_on_issuer_kind_and_last4() {
    let accounts = [
        identity(1, "ICICI Bank", CreditCard, "2003"),
        identity(2, "ICICI Bank", Savings, "2003"),
    ];
    assert_eq!(
        find(
            Some("ICICI Bank"),
            Some(CreditCard),
            Some("4123XXXXXXXX2003"),
            &accounts
        ),
        Match::One(1)
    );
}

#[test]
fn issuer_compares_ignoring_case_and_spacing() {
    let accounts = [identity(1, "SBI  Card", CreditCard, "2003")];
    assert_eq!(
        find(
            Some("sbi card"),
            Some(CreditCard),
            Some("XX2003"),
            &accounts
        ),
        Match::One(1)
    );
}

#[test]
fn a_different_last4_issuer_or_kind_is_no_match() {
    let accounts = [identity(1, "ICICI Bank", CreditCard, "2003")];
    assert_eq!(
        find(
            Some("ICICI Bank"),
            Some(CreditCard),
            Some("XX1003"),
            &accounts
        ),
        Match::None
    );
    assert_eq!(
        find(
            Some("HDFC Bank"),
            Some(CreditCard),
            Some("XX2003"),
            &accounts
        ),
        Match::None
    );
    assert_eq!(
        find(Some("ICICI Bank"), Some(Loan), Some("XX2003"), &accounts),
        Match::None
    );
}

#[test]
fn an_incomplete_proposal_matches_nothing() {
    let accounts = [identity(1, "ICICI Bank", CreditCard, "2003")];
    assert_eq!(
        find(None, Some(CreditCard), Some("XX2003"), &accounts),
        Match::None
    );
    assert_eq!(
        find(Some("ICICI Bank"), None, Some("XX2003"), &accounts),
        Match::None
    );
}

#[test]
fn a_remembered_identifier_matches_its_account() {
    let accounts = [
        identity(1, "ICICI Bank", Savings, "1111"),
        identity(1, "ICICI Bank", Savings, "2222"),
    ];
    assert_eq!(
        find(Some("ICICI Bank"), Some(Savings), Some("XX2222"), &accounts),
        Match::One(1)
    );
}

#[test]
fn several_accounts_are_reported_once_each() {
    let accounts = [
        identity(1, "SBI", Loan, "5555"),
        identity(1, "SBI", Loan, "5555"),
        identity(2, "SBI", Loan, "5555"),
    ];
    assert_eq!(
        find(Some("SBI"), Some(Loan), Some("XX5555"), &accounts),
        Match::Several(vec![1, 2])
    );
}

#[test]
fn visible_digits_are_the_trailing_digits_as_printed() {
    assert_eq!(visible_digits("XXXX XXXX XXXX XX56"), "56");
    assert_eq!(visible_digits("20334174321"), "20334174321");
    assert_eq!(visible_digits("1234XXXX"), "");
}

#[test]
fn masks_compare_on_the_shorter_run_of_trailing_digits() {
    assert!(same_number("XXXX XXXX XXXX XX56", "XXXX XXXX XXXX XX56"));
    assert!(same_number("20334174321", "XXXXXXX4321"));
    assert!(same_number("xxxx xxxx xxxx 9876", "XXXXXXXXXXXX9876"));
    assert!(same_number("XX56", "4315XXXXXXXX7056"));
}

#[test]
fn masks_with_different_trailing_digits_differ() {
    assert!(!same_number("XXXXXX4321", "XXXXXX3679"));
    assert!(!same_number("XX56", "XX35"));
}

#[test]
fn a_single_visible_digit_needs_equal_normalised_masks() {
    assert!(same_number("xx x5", "XXX5"));
    assert!(!same_number("XXXX5", "XXXXXX5"));
    assert!(!same_number("XXXX5", "XXXX6"));
}

#[test]
fn a_two_digit_mask_matches_its_own_account() {
    let accounts = [identity(1, "SBI Card", CreditCard, "XXXX XXXX XXXX XX56")];
    assert_eq!(
        find(
            Some("SBI Card"),
            Some(CreditCard),
            Some("XXXX XXXX XXXX XX56"),
            &accounts
        ),
        Match::One(1)
    );
}

#[test]
fn no_printed_number_matches_the_only_account_of_that_issuer_and_kind() {
    let accounts = [
        identity(1, "Uni", CreditCard, "XXXX9876"),
        identity(2, "SBI", Savings, "XXXXXX4321"),
    ];
    assert_eq!(
        find(Some("Uni"), Some(CreditCard), None, &accounts),
        Match::OneWithoutNumber(1)
    );
}

#[test]
fn no_printed_number_with_several_accounts_is_ambiguous() {
    let accounts = [
        identity(1, "Uni", CreditCard, "XXXX9876"),
        identity(2, "Uni", CreditCard, "XXXX2468"),
    ];
    assert_eq!(
        find(Some("Uni"), Some(CreditCard), Some("  "), &accounts),
        Match::Several(vec![1, 2])
    );
}

#[test]
fn no_printed_number_finds_an_account_that_has_no_number_either() {
    let accounts = [AccountIdentity {
        account_id: 1,
        issuer: "Uni".to_owned(),
        kind: CreditCard,
        mask: None,
    }];
    assert_eq!(
        find(Some("Uni"), Some(CreditCard), None, &accounts),
        Match::OneWithoutNumber(1)
    );
    assert_eq!(
        find(Some("Uni"), Some(CreditCard), Some("XX2003"), &accounts),
        Match::None
    );
}
