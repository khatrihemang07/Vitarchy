use super::{printing, row};
use crate::printings::{Plan, gain::would_gain, plan_for};

#[test]
fn a_printing_gains_detail_only_when_it_changes_the_shown_view() {
    let mut stored = printing("pdf text cut");
    stored.printed.balance_paise = Some(1);
    let mut richer = printing("pdf text cut off at fifty");
    richer.printed.balance_paise = Some(1);
    assert!(would_gain(std::slice::from_ref(&stored), &richer));
    assert!(!would_gain(&[stored.clone()], &printing("pdf")));
    let mut with_reference = printing("pdf text cut");
    with_reference.printed.reference = Some("R".into());
    assert!(would_gain(&[stored], &with_reference));
}

#[test]
fn a_row_adds_a_printing_only_from_a_new_statement_and_not_when_identical() {
    let stored = [printing("coffee")];
    let richer = row("coffee", Some("R1"));
    assert_eq!(
        plan_for(2, &richer, Some(&stored)),
        Plan::Add { gains: true },
        "a new reference"
    );
    assert_eq!(
        plan_for(2, &row("coffee", None), Some(&stored)),
        Plan::IdenticalKept,
        "exactly the kept Printing"
    );
    assert_eq!(
        plan_for(1, &richer, Some(&stored)),
        Plan::ThisStatement,
        "statement 1 already printed it"
    );
    let mut gone_first = printing("coffee");
    gone_first.gone = true;
    assert_eq!(
        plan_for(2, &row("coffee", None), Some(&[gone_first])),
        Plan::Add { gains: false },
        "a gone Printing does not make a row identical"
    );
}
