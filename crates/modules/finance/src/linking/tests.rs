use chrono::NaiveDate;

use super::{RecordFacts, TransactionFacts, link};
use crate::reader::contract::Direction::{self, Credit, Debit};

fn day(d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 7, d).unwrap()
}

fn record(id: i64, d: u32, amount: i64, direction: Direction, utr: Option<&str>) -> RecordFacts {
    RecordFacts {
        id,
        on: day(d),
        amount_paise: amount,
        direction,
        utr: utr.map(str::to_owned),
        instrument_issuer: None,
        instrument_digits: Some("4321".to_owned()),
    }
}

fn tx(
    id: i64,
    d: u32,
    amount: i64,
    direction: Direction,
    reference: Option<&str>,
    description: &str,
) -> TransactionFacts {
    TransactionFacts {
        id,
        date: day(d),
        amount_paise: amount,
        direction,
        reference: reference.map(str::to_owned),
        description: description.to_owned(),
        account_issuer: "SBI".to_owned(),
        account_mask: Some("XXXXXX4321".to_owned()),
    }
}

#[test]
fn links_by_utr_equal_to_the_reference_even_when_amounts_and_dates_differ() {
    let links = link(
        &[record(1, 5, 100, Debit, Some("512345678901"))],
        &[tx(10, 20, 999, Debit, Some("512345678901"), "UPI")],
    );
    assert_eq!(links, vec![(1, 10)]);
}

#[test]
fn links_by_utr_found_in_the_description() {
    let links = link(
        &[record(1, 5, 100, Debit, Some("512345678901"))],
        &[tx(10, 5, 100, Debit, None, "UPI/DR/512345678901/SHOP")],
    );
    assert_eq!(links, vec![(1, 10)]);
}

#[test]
fn falls_back_to_amount_direction_and_a_date_within_a_day() {
    let transactions = [
        tx(10, 6, 100, Debit, None, "x"),
        tx(11, 9, 100, Debit, None, "y"),
    ];
    assert_eq!(
        link(&[record(1, 5, 100, Debit, None)], &transactions),
        vec![(1, 10)]
    );
    assert!(link(&[record(1, 3, 100, Debit, None)], &transactions).is_empty());
}

#[test]
fn the_nearest_date_wins_the_fallback() {
    let transactions = [
        tx(10, 4, 100, Debit, None, "x"),
        tx(11, 5, 100, Debit, None, "y"),
    ];
    assert_eq!(
        link(&[record(1, 5, 100, Debit, None)], &transactions),
        vec![(1, 11)]
    );
}

#[test]
fn a_transaction_takes_at_most_one_record() {
    let links = link(
        &[
            record(1, 5, 100, Debit, None),
            record(2, 5, 100, Debit, None),
        ],
        &[tx(10, 5, 100, Debit, None, "x")],
    );
    assert_eq!(links, vec![(1, 10)]);
}

#[test]
fn a_refund_sharing_the_original_utr_links_by_direction() {
    let records = [
        record(1, 5, 100, Debit, Some("777777777777")),
        record(2, 6, 100, Credit, Some("777777777777")),
    ];
    let transactions = [
        tx(10, 6, 100, Credit, Some("777777777777"), "refund"),
        tx(11, 5, 100, Debit, Some("777777777777"), "payment"),
    ];
    let mut links = link(&records, &transactions);
    links.sort_unstable();
    assert_eq!(links, vec![(1, 11), (2, 10)]);
}

#[test]
fn utr_matches_are_claimed_before_amount_matches() {
    // Record 1 would take tx 10 by amount, but record 2's UTR owns it.
    let records = [
        record(1, 5, 100, Debit, None),
        record(2, 5, 100, Debit, Some("111111111111")),
    ];
    let transactions = [
        tx(10, 5, 100, Debit, Some("111111111111"), "x"),
        tx(11, 5, 100, Debit, None, "y"),
    ];
    let mut links = link(&records, &transactions);
    links.sort_unstable();
    assert_eq!(links, vec![(1, 11), (2, 10)]);
}

#[test]
fn different_references_are_different_payments() {
    let links = link(
        &[record(1, 5, 100, Debit, Some("111111111111"))],
        &[tx(10, 5, 100, Debit, Some("222222222222"), "x")],
    );
    assert!(links.is_empty());
}

#[test]
fn a_blank_utr_is_ignored_and_direction_must_agree_for_amount_matches() {
    let transactions = [tx(10, 5, 100, Credit, None, "anything")];
    assert!(link(&[record(1, 5, 100, Debit, Some("  "))], &transactions).is_empty());
}

fn instrument(mut record: RecordFacts, issuer: Option<&str>, digits: Option<&str>) -> RecordFacts {
    record.instrument_issuer = issuer.map(str::to_owned);
    record.instrument_digits = digits.map(str::to_owned);
    record
}

fn account(mut tx: TransactionFacts, issuer: &str, mask: Option<&str>) -> TransactionFacts {
    tx.account_issuer = issuer.to_owned();
    tx.account_mask = mask.map(str::to_owned);
    tx
}

#[test]
fn a_record_never_links_to_an_account_with_other_trailing_digits() {
    let records = [record(1, 5, 100, Debit, Some("123456789012"))];
    let other = account(
        tx(10, 5, 100, Debit, Some("123456789012"), "x"),
        "SBI",
        Some("XXXXXX9999"),
    );
    assert!(link(&records, &[other]).is_empty());
}

#[test]
fn the_issuer_must_agree_when_the_record_names_one() {
    let records = [instrument(
        record(1, 5, 100, Debit, None),
        Some("ICICI Bank"),
        Some("8765"),
    )];
    let sbi = account(tx(10, 5, 100, Debit, None, "x"), "SBI", Some("XXXXXX8765"));
    let icici = account(
        tx(11, 5, 100, Debit, None, "y"),
        "icici  bank",
        Some("XXXXXX8765"),
    );
    assert_eq!(link(&records, &[sbi, icici]), vec![(1, 11)]);
}

#[test]
fn an_issuer_alone_is_enough_to_name_the_instrument() {
    let records = [instrument(
        record(1, 5, 100, Debit, None),
        Some("ICICI Bank"),
        None,
    )];
    let icici = account(tx(10, 5, 100, Debit, None, "x"), "ICICI Bank", None);
    assert_eq!(link(&records, &[icici]), vec![(1, 10)]);
}

#[test]
fn digits_cannot_agree_with_an_account_that_has_no_number() {
    let records = [record(1, 5, 100, Debit, None)];
    let numberless = account(tx(10, 5, 100, Debit, None, "x"), "SBI", None);
    assert!(link(&records, &[numberless]).is_empty());
}

#[test]
fn an_unknown_instrument_links_by_utr_only() {
    let records = [
        instrument(record(1, 5, 100, Debit, None), None, None),
        instrument(record(2, 5, 200, Debit, Some("555555555555")), None, None),
    ];
    let transactions = [
        tx(10, 5, 100, Debit, None, "same amount and day, no UTR"),
        tx(11, 9, 200, Debit, Some("555555555555"), "UTR matches"),
    ];
    assert_eq!(link(&records, &transactions), vec![(2, 11)]);
}

#[test]
fn amount_matching_stays_inside_the_agreeing_account() {
    let records = [record(1, 5, 100, Debit, None)];
    let other = account(tx(10, 5, 100, Debit, None, "x"), "SBI", Some("XXXXXX1111"));
    let mine = tx(11, 6, 100, Debit, None, "y");
    assert_eq!(link(&records, &[other, mine]), vec![(1, 11)]);
}

#[test]
fn a_utr_inside_a_longer_digit_run_in_the_description_is_not_a_match() {
    for description in [
        "UPI/DR/9512345678901/SHOP",
        "UPI/DR/5123456789012/SHOP",
        "REF 3512345678901234",
    ] {
        let links = link(
            &[record(1, 5, 100, Debit, Some("512345678901"))],
            &[tx(10, 5, 999, Debit, None, description)],
        );
        assert_eq!(links, vec![], "{description}");
    }
}

#[test]
fn a_utr_set_off_by_non_digits_in_the_description_is_a_match() {
    for description in [
        "512345678901",
        "UPI-512345678901-SHOP",
        "ref:512345678901.",
        "9999 512345678901 9999",
    ] {
        let links = link(
            &[record(1, 5, 100, Debit, Some("512345678901"))],
            &[tx(10, 5, 999, Debit, None, description)],
        );
        assert_eq!(links, vec![(1, 10)], "{description}");
    }
}

#[test]
fn a_later_whole_occurrence_matches_after_an_embedded_one() {
    let links = link(
        &[record(1, 5, 100, Debit, Some("512345678901"))],
        &[tx(10, 5, 999, Debit, None, "A9512345678901 B512345678901")],
    );
    assert_eq!(links, vec![(1, 10)]);
}
