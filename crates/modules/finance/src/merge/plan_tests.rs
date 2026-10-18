use super::plan::{Move, Row, plan};
use crate::test_support::tx;

fn row(id: i64, description: &str, amount: i64) -> Row {
    Row {
        id,
        tx: tx(description, amount),
    }
}

#[test]
fn a_row_the_kept_account_lacks_moves_over() {
    let moves = plan(&[row(1, "coffee shop", 100)], &[row(7, "book store", 250)]);
    assert_eq!(moves, vec![Move::Repoint { id: 7 }]);
}

#[test]
fn a_row_the_kept_account_has_folds_into_it() {
    let moves = plan(&[row(1, "coffee shop", 100)], &[row(7, "coffee shop", 100)]);
    assert_eq!(
        moves,
        vec![Move::Fold {
            from_id: 7,
            into_id: 1
        }]
    );
}

#[test]
fn one_kept_row_absorbs_only_one_of_two_identical_rows() {
    let moves = plan(
        &[row(1, "coffee shop", 100)],
        &[row(7, "coffee shop", 100), row(8, "coffee shop", 100)],
    );
    assert_eq!(
        moves,
        vec![
            Move::Fold {
                from_id: 7,
                into_id: 1
            },
            Move::Repoint { id: 8 }
        ]
    );
}

#[test]
fn nothing_to_merge_plans_nothing() {
    assert!(plan(&[row(1, "a", 1)], &[]).is_empty());
}
