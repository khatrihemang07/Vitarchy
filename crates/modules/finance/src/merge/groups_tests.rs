use super::groups::{Group, duplicate_groups};
use crate::{
    accounts::AccountIdentity,
    reader::contract::AccountKind::{self, CreditCard, Loan, Savings},
};

fn account(id: i64, issuer: &str, kind: AccountKind, mask: Option<&str>) -> AccountIdentity {
    AccountIdentity {
        account_id: id,
        issuer: issuer.to_owned(),
        kind,
        mask: mask.map(str::to_owned),
    }
}

#[test]
fn the_oldest_of_each_group_is_kept() {
    let accounts = [
        account(1, "SBI", Savings, Some("XXXXXX7391")),
        account(2, "SBI", Savings, Some("XXXXXX7391")),
        account(3, "SBI", Loan, Some("XXXXXX7391")),
        account(4, "SBI", Savings, Some("XX7391")),
    ];
    assert_eq!(
        duplicate_groups(&accounts),
        vec![Group {
            keep: 1,
            from: vec![2, 4]
        }]
    );
}

#[test]
fn issuers_match_ignoring_case_and_spacing() {
    let accounts = [
        account(1, "ICICI Bank", CreditCard, Some("4123XXXXXXXX2003")),
        account(2, "icici  bank", CreditCard, Some("XXXX2003")),
    ];
    assert_eq!(duplicate_groups(&accounts)[0].from, vec![2]);
}

#[test]
fn different_numbers_or_kinds_are_different_accounts() {
    let accounts = [
        account(1, "SBI", Savings, Some("XXXX7391")),
        account(2, "SBI", Savings, Some("XXXX1111")),
        account(3, "SBI", CreditCard, Some("XXXX7391")),
    ];
    assert!(duplicate_groups(&accounts).is_empty());
}

#[test]
fn accounts_without_a_number_are_never_grouped() {
    let accounts = [
        account(1, "SBI", Savings, None),
        account(2, "SBI", Savings, None),
    ];
    assert!(duplicate_groups(&accounts).is_empty());
}
