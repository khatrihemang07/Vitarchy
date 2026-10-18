use std::collections::HashMap;

use super::{
    AccountIdentity, Match, find_for,
    instrument::{find_instrument, instrument_key},
};
use crate::reader::contract::{
    AccountKind::{self, CreditCard, Savings},
    Kind, ProposedAccount,
};

fn identity(id: i64, issuer: &str, kind: AccountKind, mask: Option<&str>) -> AccountIdentity {
    AccountIdentity {
        account_id: id,
        issuer: issuer.to_owned(),
        kind,
        mask: mask.map(str::to_owned),
    }
}

fn instrument(issuer: Option<&str>, mask: Option<&str>) -> ProposedAccount {
    ProposedAccount {
        issuer: issuer.map(str::to_owned),
        kind: None,
        mask: mask.map(str::to_owned),
    }
}

fn none() -> HashMap<String, i64> {
    HashMap::new()
}

#[test]
fn an_instrument_is_remembered_ignoring_case_and_spacing() {
    assert_eq!(
        instrument_key(Some(" icici   r ")).as_deref(),
        Some("ICICI R")
    );
    assert_eq!(instrument_key(Some("  ")), None);
    assert_eq!(instrument_key(None), None);
}

#[test]
fn a_remembered_instrument_wins_over_every_other_match() {
    let accounts = [
        identity(1, "Bank A", Savings, Some("XXXXXX4321")),
        identity(2, "Bank B", CreditCard, Some("XXXX9999")),
    ];
    let remembered = HashMap::from([("XXXXXX4321".to_owned(), 2)]);
    assert_eq!(
        find_instrument(
            &instrument(None, Some("XXXXXX4321")),
            &remembered,
            &accounts
        ),
        Match::One(2)
    );
}

#[test]
fn a_remembered_account_that_no_longer_exists_falls_through() {
    let accounts = [identity(1, "Bank A", Savings, Some("XXXXXX4321"))];
    let remembered = HashMap::from([("XXXXXX4321".to_owned(), 77)]);
    assert_eq!(
        find_instrument(
            &instrument(None, Some("XXXXXX4321")),
            &remembered,
            &accounts
        ),
        Match::One(1)
    );
}

#[test]
fn visible_digits_match_across_kinds_when_the_instrument_names_no_issuer() {
    let accounts = [
        identity(1, "Bank A", Savings, Some("XXXXXX4321")),
        identity(2, "Bank A", Savings, Some("XXXXXX1111")),
    ];
    assert_eq!(
        find_instrument(&instrument(None, Some("XXXXXX4321")), &none(), &accounts),
        Match::One(1)
    );
    let card = [identity(5, "Bank B", CreditCard, Some("4123XXXXXXXX4321"))];
    assert_eq!(
        find_instrument(&instrument(None, Some("XXXXXX4321")), &none(), &card),
        Match::One(5),
        "a card matches an instrument that cannot say it is a card"
    );
}

#[test]
fn the_issuer_must_agree_when_the_instrument_names_one() {
    let accounts = [
        identity(1, "Bank A", Savings, Some("XXXXXX4321")),
        identity(2, "Bank B", Savings, Some("XXXXXX4321")),
    ];
    assert_eq!(
        find_instrument(
            &instrument(Some("bank  b"), Some("XXXXXX4321")),
            &none(),
            &accounts
        ),
        Match::One(2)
    );
}

#[test]
fn several_accounts_with_the_same_digits_are_all_candidates() {
    let accounts = [
        identity(1, "Bank A", Savings, Some("XXXXXX4321")),
        identity(2, "Bank B", CreditCard, Some("XXXX4321")),
    ];
    assert_eq!(
        find_instrument(&instrument(None, Some("XXXXXX4321")), &none(), &accounts),
        Match::Several(vec![1, 2])
    );
}

#[test]
fn an_instrument_with_no_digits_matches_by_issuer_and_flags_the_missing_number() {
    let one = [identity(
        1,
        "ICICI Bank",
        CreditCard,
        Some("4123XXXXXXXX2003"),
    )];
    assert_eq!(
        find_instrument(
            &instrument(Some("ICICI Bank"), Some("ICICI R")),
            &none(),
            &one
        ),
        Match::OneWithoutNumber(1)
    );
    let two = [
        identity(1, "ICICI Bank", CreditCard, Some("4123XXXXXXXX2003")),
        identity(2, "ICICI Bank", Savings, Some("XXXX6666")),
    ];
    assert_eq!(
        find_instrument(
            &instrument(Some("ICICI Bank"), Some("ICICI R")),
            &none(),
            &two
        ),
        Match::Several(vec![1, 2])
    );
}

#[test]
fn an_instrument_with_neither_digits_nor_issuer_matches_nothing() {
    let accounts = [identity(1, "Bank A", Savings, Some("XXXXXX4321"))];
    assert_eq!(
        find_instrument(&instrument(None, Some("BOB R")), &none(), &accounts),
        Match::None
    );
    assert_eq!(
        find_instrument(&instrument(None, None), &none(), &accounts),
        Match::None
    );
}

#[test]
fn other_digits_or_a_missing_number_do_not_match() {
    let accounts = [
        identity(1, "Bank A", Savings, Some("XXXXXX1111")),
        identity(2, "Bank A", Savings, None),
    ];
    assert_eq!(
        find_instrument(&instrument(None, Some("XXXXXX4321")), &none(), &accounts),
        Match::None
    );
}

#[test]
fn a_statement_still_matches_by_issuer_kind_and_number() {
    let accounts = [
        identity(1, "Bank A", Savings, Some("XXXXXX4321")),
        identity(2, "Bank A", CreditCard, Some("XXXXXX4321")),
    ];
    let proposal = ProposedAccount {
        issuer: Some("Bank A".to_owned()),
        kind: Some(CreditCard),
        mask: Some("XXXXXX4321".to_owned()),
    };
    assert_eq!(
        find_for(Some(Kind::Statement), &proposal, &none(), &accounts),
        Match::One(2)
    );
    assert_eq!(
        find_for(Some(Kind::PaymentHistory), &proposal, &none(), &accounts),
        Match::Several(vec![1, 2])
    );
}
