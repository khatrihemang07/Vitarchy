use super::{Printing, gain::would_gain, gains_detail, resolve};
use crate::reader::contract::{Direction, Transaction};

fn printing(description: &str) -> Printing {
    Printing {
        statement_id: 1,
        description: description.to_owned(),
        balance_paise: None,
        reference: None,
        source_id: None,
        foreign_amount: None,
        foreign_currency: None,
        gone: false,
    }
}

fn gone(mut p: Printing) -> Printing {
    p.gone = true;
    p
}

#[test]
fn a_later_description_that_extends_the_shown_one_replaces_it() {
    let shown = resolve(&[
        printing("UPI/Acme Store/123"),
        printing("UPI/Acme Store/123/payment for order 9"),
    ]);
    assert_eq!(
        shown.detail.description,
        "UPI/Acme Store/123/payment for order 9"
    );
}

#[test]
fn extension_ignores_case_spacing_and_punctuation() {
    let shown = resolve(&[
        printing("neft cr - acme"),
        printing("NEFT CR ACME CORP salary"),
    ]);
    assert_eq!(shown.detail.description, "NEFT CR ACME CORP salary");
}

#[test]
fn a_later_shorter_description_keeps_the_first() {
    let shown = resolve(&[
        printing("UPI/Acme Store/123/order 9"),
        printing("UPI/Acme Store"),
    ]);
    assert_eq!(shown.detail.description, "UPI/Acme Store/123/order 9");
}

#[test]
fn an_unrelated_description_keeps_the_first() {
    let other = resolve(&[printing("rent march"), printing("electricity bill payment")]);
    assert_eq!(other.detail.description, "rent march");
}

#[test]
fn an_equal_normalised_description_keeps_the_first_text() {
    let shown = resolve(&[printing("Upi Shop"), printing("UPI-SHOP")]);
    assert_eq!(shown.detail.description, "Upi Shop");
}

#[test]
fn the_shown_description_grows_step_by_step() {
    let shown = resolve(&[printing("ab"), printing("abc"), printing("abcd")]);
    assert_eq!(shown.detail.description, "abcd");
}

#[test]
fn gaps_are_filled_from_the_first_printing_that_has_a_value() {
    let mut a = printing("x");
    a.reference = Some("R1".into());
    let mut b = printing("x");
    b.balance_paise = Some(900);
    b.reference = Some("R2".into());
    b.source_id = Some("S9".into());
    b.foreign_amount = Some("5.00".into());
    b.foreign_currency = Some("USD".into());
    let shown = resolve(&[a, b]).detail;
    assert_eq!(
        (
            shown.balance_paise,
            shown.reference.as_deref(),
            shown.source_id.as_deref()
        ),
        (Some(900), Some("R1"), Some("S9"))
    );
    assert_eq!(
        (
            shown.foreign_amount.as_deref(),
            shown.foreign_currency.as_deref()
        ),
        (Some("5.00"), Some("USD"))
    );
}

#[test]
fn gone_printings_are_ignored_while_a_live_one_exists() {
    let mut full = printing("a much fuller description");
    full.balance_paise = Some(5);
    let shown = resolve(&[gone(full), printing("a much")]);
    assert_eq!(shown.detail.description, "a much");
    assert_eq!(shown.detail.balance_paise, None);
    assert!(!shown.no_longer_found);
}

#[test]
fn a_transaction_is_no_longer_found_only_when_every_printing_is_gone() {
    let some = resolve(&[gone(printing("a")), printing("a")]);
    let all = resolve(&[gone(printing("a")), gone(printing("a longer one"))]);
    assert!(!some.no_longer_found);
    assert!(all.no_longer_found);
    assert_eq!(
        all.detail.description, "a longer one",
        "all gone: all still count"
    );
}

#[test]
fn a_printing_gains_detail_only_when_it_changes_the_shown_view() {
    let mut stored = printing("pdf text cut");
    stored.balance_paise = Some(1);
    let mut richer = printing("pdf text cut off at fifty");
    richer.balance_paise = Some(1);
    assert!(would_gain(std::slice::from_ref(&stored), &richer));
    assert!(!would_gain(&[stored.clone()], &printing("pdf")));
    let mut with_reference = printing("pdf text cut");
    with_reference.reference = Some("R".into());
    assert!(would_gain(&[stored], &with_reference));
}

fn row(description: &str, reference: Option<&str>) -> Transaction {
    Transaction {
        date: chrono::NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
        description: description.to_owned(),
        amount_paise: 100,
        direction: Direction::Debit,
        balance_paise: None,
        foreign_amount: None,
        foreign_currency: None,
        reference: reference.map(str::to_owned),
        source_id: None,
    }
}

#[test]
fn a_row_gains_detail_only_from_a_new_statement_that_adds_something() {
    let stored = [printing("coffee")];
    let richer = row("coffee", Some("R1"));
    assert!(gains_detail(2, &richer, Some(&stored)), "a new reference");
    assert!(
        !gains_detail(2, &row("coffee", None), Some(&stored)),
        "nothing new"
    );
    assert!(
        !gains_detail(1, &richer, Some(&stored)),
        "statement 1 already printed it"
    );
    assert!(
        !gains_detail(2, &richer, None),
        "no stored Transaction to gain"
    );
}
