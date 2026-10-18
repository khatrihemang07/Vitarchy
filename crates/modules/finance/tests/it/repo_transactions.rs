use crate::common::{account, date, finance_db, statement, tx};
use vitarchy_finance::{new_rows, read_again_diff, repo::transactions};

#[tokio::test]
async fn stored_rows_come_back_as_evidence_that_drives_dedupe() {
    let db = finance_db("tx_dedupe").await;
    let account = account(&db, "XX2003").await;
    let statement = statement(&db, 'a').await;
    let first = [tx(1, "Coffee", 100), tx(2, "Tea", 200)];
    for row in &first {
        transactions::insert(&db.pool, account.id, statement, row)
            .await
            .unwrap();
    }
    let stored = transactions::evidence(&db.pool, account.id).await.unwrap();
    let second = [
        tx(1, "coffee", 100),
        tx(2, "Tea", 200),
        tx(2, "Tea", 200),
        tx(3, "New", 300),
    ];
    let fresh = new_rows(&second, &stored);
    assert_eq!(fresh, vec![2, 3]);
    db.drop().await;
}

#[tokio::test]
async fn read_again_marks_lost_rows_and_clearing_restores_them_without_deleting() {
    let db = finance_db("tx_lost").await;
    let account = account(&db, "XX2003").await;
    let statement = statement(&db, 'a').await;
    let kept = tx(1, "Kept", 100);
    transactions::insert(&db.pool, account.id, statement, &kept)
        .await
        .unwrap();
    transactions::insert(&db.pool, account.id, statement, &tx(2, "Gone", 200))
        .await
        .unwrap();

    let keys = transactions::stored_keys(&db.pool, account.id)
        .await
        .unwrap();
    let diff = read_again_diff(std::slice::from_ref(&kept), &keys);
    assert_eq!(diff.no_longer_found.len(), 1);
    assert_eq!(
        transactions::mark_no_longer_found(&db.pool, &diff.no_longer_found)
            .await
            .unwrap(),
        1
    );

    let rows = transactions::list(&db.pool, account.id, None, None)
        .await
        .unwrap();
    assert_eq!(rows.len(), 2, "nothing is deleted");
    assert_eq!(
        rows.iter()
            .filter(|r| r.no_longer_found_at.is_some())
            .count(),
        1
    );
    let again = read_again_diff(
        &[kept, tx(2, "Gone", 200)],
        &transactions::stored_keys(&db.pool, account.id)
            .await
            .unwrap(),
    );
    assert!(again.add.is_empty() && again.no_longer_found.is_empty());
    transactions::clear_no_longer_found(&db.pool, &again.unchanged)
        .await
        .unwrap();
    let rows = transactions::list(&db.pool, account.id, None, None)
        .await
        .unwrap();
    assert!(rows.iter().all(|r| r.no_longer_found_at.is_none()));
    db.drop().await;
}

#[tokio::test]
async fn list_filters_by_date_newest_first() {
    let db = finance_db("tx_list").await;
    let account = account(&db, "XX2003").await;
    let statement = statement(&db, 'a').await;
    for day in [1, 5, 9] {
        transactions::insert(&db.pool, account.id, statement, &tx(day, "x", 100))
            .await
            .unwrap();
    }
    let rows = transactions::list(&db.pool, account.id, Some(date(2)), Some(date(9)))
        .await
        .unwrap();
    assert_eq!(
        rows.iter().map(|r| r.date).collect::<Vec<_>>(),
        [date(9), date(5)]
    );
    db.drop().await;
}

#[tokio::test]
async fn a_source_id_and_balance_are_stored_with_the_row() {
    let db = finance_db("tx_source").await;
    let account = account(&db, "XX2003").await;
    let statement = statement(&db, 'a').await;
    let mut row = tx(1, "UPI", 100);
    row.source_id = Some("T1".into());
    row.balance_paise = Some(-5);
    row.reference = Some("512345678901".into());
    transactions::insert(&db.pool, account.id, statement, &row)
        .await
        .unwrap();
    let stored = &transactions::list(&db.pool, account.id, None, None)
        .await
        .unwrap()[0];
    assert_eq!(
        (stored.source_id.as_deref(), stored.balance_paise),
        (Some("T1"), Some(-5))
    );
    assert_eq!(stored.reference.as_deref(), Some("512345678901"));
    db.drop().await;
}
