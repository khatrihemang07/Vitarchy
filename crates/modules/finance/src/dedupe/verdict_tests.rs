use chrono::Days;

use super::{
    Evidence,
    verdict::{Rung, verdict},
};
use crate::{reader::contract::Direction, test_support::tx};

fn ev(description: &str) -> Evidence {
    Evidence::from(&tx(description, 100))
}

fn with(mut e: Evidence, change: impl FnOnce(&mut Evidence)) -> Evidence {
    change(&mut e);
    e
}

fn rung(a: &Evidence, b: &Evidence) -> Option<Rung> {
    verdict(a, b).map(|v| v.rung)
}

#[test]
fn equal_descriptions_match_ignoring_case_spacing_and_punctuation() {
    assert_eq!(
        rung(&ev("UPI/DR  Shop\n One"), &ev("upi dr-shop one")),
        Some(Rung::Description)
    );
}

#[test]
fn amount_or_direction_differences_never_match() {
    let base = ev("coffee shop visit");
    assert_eq!(
        rung(&base, &with(base.clone(), |e| e.amount_paise = 101)),
        None
    );
    assert_eq!(
        rung(
            &base,
            &with(base.clone(), |e| e.direction = Direction::Credit)
        ),
        None
    );
}

#[test]
fn balance_decides_across_differing_descriptions() {
    let a = with(ev("neft cr acme corp salary for the month of june"), |e| {
        e.balance_paise = Some(500);
    });
    let b = with(ev("neft cr acme"), |e| e.balance_paise = Some(500));
    assert_eq!(rung(&a, &b), Some(Rung::Balance));
}

#[test]
fn a_different_balance_means_not_the_same_even_with_equal_descriptions() {
    let a = with(ev("coffee shop"), |e| e.balance_paise = Some(500));
    let b = with(ev("coffee shop"), |e| e.balance_paise = Some(600));
    assert_eq!(rung(&a, &b), None);
}

#[test]
fn one_missing_balance_falls_through_to_the_description() {
    let a = with(ev("coffee shop"), |e| e.balance_paise = Some(500));
    assert_eq!(rung(&a, &ev("coffee shop")), Some(Rung::Description));
}

#[test]
fn a_day_apart_matches_only_with_a_balance_or_an_id() {
    let later = |e: &mut Evidence| e.date = e.date.checked_add_days(Days::new(1)).unwrap();
    assert_eq!(
        rung(
            &ev("coffee shop visit"),
            &with(ev("coffee shop visit"), later)
        ),
        None
    );
    let balanced = with(ev("x"), |e| e.balance_paise = Some(9));
    assert_eq!(
        rung(&balanced, &with(balanced.clone(), later)),
        Some(Rung::Balance)
    );
    let ided = with(ev("x"), |e| e.source_id = Some("T1".to_owned()));
    assert_eq!(
        rung(&ided, &with(ided.clone(), later)),
        Some(Rung::SourceId)
    );
    let referenced = with(ev("x"), |e| e.reference = Some("R1".to_owned()));
    assert_eq!(rung(&referenced, &with(referenced.clone(), later)), None);
}

#[test]
fn two_days_apart_never_match() {
    let far = |e: &mut Evidence| e.date = e.date.checked_add_days(Days::new(2)).unwrap();
    let ided = with(ev("x"), |e| e.source_id = Some("T1".to_owned()));
    assert_eq!(rung(&ided, &with(ided.clone(), far)), None);
}

#[test]
fn a_source_id_decides_even_when_everything_else_differs() {
    let a = with(ev("one"), |e| e.source_id = Some("T42".to_owned()));
    let b = with(ev("completely different"), |e| {
        e.source_id = Some("T42".to_owned());
        e.balance_paise = Some(1);
    });
    assert_eq!(rung(&a, &b), Some(Rung::SourceId));
}

#[test]
fn different_source_ids_block_despite_equal_descriptions() {
    let a = with(ev("coffee shop"), |e| e.source_id = Some("T1".to_owned()));
    let b = with(ev("coffee shop"), |e| e.source_id = Some("T2".to_owned()));
    assert_eq!(rung(&a, &b), None);
}

#[test]
fn a_reference_disagreement_blocks_equal_descriptions() {
    let a = with(ev("upi shop"), |e| e.reference = Some("111".to_owned()));
    let b = with(ev("upi shop"), |e| e.reference = Some("222".to_owned()));
    assert_eq!(rung(&a, &b), None);
    let same = with(ev("something else"), |e| {
        e.reference = Some("111".to_owned());
    });
    assert_eq!(rung(&a, &same), Some(Rung::Reference));
}

#[test]
fn a_long_enough_prefix_matches_a_short_one_does_not() {
    assert_eq!(
        rung(
            &ev("neft cr acme corp"),
            &ev("neft cr acme corp salary june")
        ),
        Some(Rung::Description)
    );
    // 7 characters: below the prefix minimum and too dissimilar overall.
    assert_eq!(
        rung(&ev("upi-abc"), &ev("upiabcxxxxxxxxxxxxxxxxxxxxxx")),
        None
    );
}

#[test]
fn similar_descriptions_match_at_the_threshold_and_dissimilar_ones_do_not() {
    assert!(
        rung(
            &ev("swiggy order 123 bangalore"),
            &ev("swiggy order 123 bangalor")
        )
        .is_some()
    );
    assert_eq!(
        rung(&ev("swiggy order 123"), &ev("demoeats order 456")),
        None
    );
}
