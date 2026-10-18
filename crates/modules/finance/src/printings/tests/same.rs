use super::printing;
use crate::printings::{Printing, identical_to_live, same::same_as};

#[test]
fn printings_are_the_same_only_when_every_printed_field_is_equal() {
    let base = printing("coffee");
    let mut other_statement = base.clone();
    other_statement.statement_id = 9;
    other_statement.gone = true;
    assert!(
        same_as(&base, &other_statement),
        "statement and gone ignored"
    );
    let changes: [fn(&mut Printing); 8] = [
        |p| p.printed.date = chrono::NaiveDate::from_ymd_opt(2026, 3, 2).unwrap(),
        |p| p.printed.time = chrono::NaiveTime::from_hms_opt(9, 30, 0),
        |p| p.printed.description.push('!'),
        |p| p.printed.balance_paise = Some(1),
        |p| p.printed.reference = Some("R".into()),
        |p| p.printed.source_id = Some("S".into()),
        |p| p.printed.foreign_amount = Some("1.00".into()),
        |p| p.printed.foreign_currency = Some("USD".into()),
    ];
    for change in changes {
        let mut different = base.clone();
        change(&mut different);
        assert!(!same_as(&base, &different));
    }
}

#[test]
fn a_merge_drops_only_printings_identical_to_a_live_one_already_kept() {
    let kept = [printing("coffee")];
    let mut same = printing("coffee");
    same.id = 2;
    let mut richer = printing("coffee");
    richer.id = 3;
    richer.printed.reference = Some("R".into());
    assert_eq!(identical_to_live(&[same.clone(), richer], &kept), vec![2]);
    let mut gone = kept[0].clone();
    gone.gone = true;
    assert!(identical_to_live(&[same], &[gone]).is_empty());
}
