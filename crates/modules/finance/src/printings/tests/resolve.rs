use super::{gone, printing};
use crate::printings::resolve;

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
    a.printed.reference = Some("R1".into());
    let mut b = printing("x");
    b.printed.balance_paise = Some(900);
    b.printed.reference = Some("R2".into());
    b.printed.source_id = Some("S9".into());
    b.printed.foreign_amount = Some("5.00".into());
    b.printed.foreign_currency = Some("USD".into());
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
    full.printed.balance_paise = Some(5);
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

fn at(printing: crate::printings::Printing, hour: u32) -> crate::printings::Printing {
    let mut p = printing;
    p.printed.time = chrono::NaiveTime::from_hms_opt(hour, 20, 0);
    p
}

#[test]
fn the_time_is_the_first_one_any_printing_prints() {
    let shown = resolve(&[
        printing("bank row"),
        at(printing("app row"), 11),
        at(printing("x"), 14),
    ]);
    assert_eq!(
        shown.detail.time,
        chrono::NaiveTime::from_hms_opt(11, 20, 0)
    );
}

#[test]
fn a_transaction_no_printing_gives_a_time_shows_none() {
    assert_eq!(resolve(&[printing("bank row")]).detail.time, None);
}

#[test]
fn a_gone_printings_time_is_ignored_while_a_live_one_exists() {
    let shown = resolve(&[gone(at(printing("app row"), 9)), printing("bank row")]);
    assert_eq!(shown.detail.time, None);
}
