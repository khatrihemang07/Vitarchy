use crate::common::{account, date, finance_db, statement, tx};
use vitarchy_finance::{
    new_rows, read_again_diff,
    repo::{printings, transactions},
};

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
    let stored: Vec<_> =
        transactions::stored_keys(&mut db.pool.acquire().await.unwrap(), account.id)
            .await
            .unwrap()
            .into_iter()
            .map(|key| key.evidence)
            .collect();
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
async fn read_again_marks_gone_printings_and_clearing_restores_them_without_deleting() {
    let db = finance_db("tx_lost").await;
    let account = account(&db, "XX2003").await;
    let statement = statement(&db, 'a').await;
    let kept = tx(1, "Kept", 100);
    let vanished = tx(2, "Gone", 200);
    for row in [&kept, &vanished] {
        let id = transactions::insert(&db.pool, account.id, statement, row)
            .await
            .unwrap();
        printings::insert(&db.pool, id, statement, row)
            .await
            .unwrap();
    }
    let diff_of = |rows: Vec<_>| {
        let pool = db.pool.clone();
        async move {
            let keys = transactions::stored_keys(&mut pool.acquire().await.unwrap(), account.id)
                .await
                .unwrap();
            let here = printings::marks_of_statement(&pool, account.id, statement)
                .await
                .unwrap();
            read_again_diff(&rows, vitarchy_finance::Origin::Statement, &keys, &here)
        }
    };

    let diff = diff_of(vec![kept.clone()]).await;
    assert_eq!(diff.gone_transaction_ids.len(), 1);
    let marked = printings::set_gone(&db.pool, statement, &diff.gone_transaction_ids, true)
        .await
        .unwrap();
    assert_eq!(marked, diff.gone_transaction_ids);

    let rows = transactions::list(&db.pool, account.id, None, None)
        .await
        .unwrap();
    assert_eq!(rows.len(), 2, "nothing is deleted");
    let again = diff_of(vec![kept, vanished]).await;
    assert!(again.add.is_empty() && again.gone_transaction_ids.is_empty());
    let cleared = printings::set_gone(&db.pool, statement, &again.unchanged, false)
        .await
        .unwrap();
    assert_eq!(cleared.len(), 1, "only the marked Printing changes");
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
