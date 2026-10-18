//! The migration that retires Payment records: linked ones become Printings, and the
//! Transaction shows what they add.

/// A converted Printing as the migration wrote it: transaction, time, description, UTR, app id.
type Converted = (i64, String, String, Option<String>, Option<String>);
/// What a Transaction shows: description, reference, source id, time.
type Shown = (String, Option<String>, Option<String>, Option<String>);

use crate::common::old_shape::{old_database, seed_two_histories};

#[tokio::test]
async fn linked_records_become_printings_with_their_time_utr_and_app_id() {
    let (db, module) = old_database("pa_mig_printings").await;
    seed_two_histories(&db).await;

    module.migrator().run(&db.pool).await.unwrap();

    let rows: Vec<Converted> = sqlx::query_as(
        "SELECT transaction_id, time::text, description, reference, source_id
         FROM finance.printings WHERE statement_id > 1 ORDER BY transaction_id",
    )
    .fetch_all(&db.pool)
    .await
    .unwrap();
    assert_eq!(
        rows,
        vec![
            (
                1,
                "11:20:00".into(),
                "Paid to SHOP".into(),
                Some("512345678901".into()),
                Some("T1".into())
            ),
            (
                2,
                "09:05:00".into(),
                "Paid to OTHER".into(),
                None,
                Some("T2".into())
            ),
            (
                3,
                "10:00:00".into(),
                "Paid to LANDLORD".into(),
                Some("U3".into()),
                Some("T3".into())
            ),
        ]
    );
    db.drop().await;
}

#[tokio::test]
async fn the_transaction_shows_what_its_new_printing_adds_and_keeps_its_own_reference() {
    let (db, module) = old_database("pa_mig_resolve").await;
    seed_two_histories(&db).await;

    module.migrator().run(&db.pool).await.unwrap();

    let shown: Vec<Shown> = sqlx::query_as(
        "SELECT description, reference, source_id, time::text FROM finance.transactions ORDER BY id",
    )
    .fetch_all(&db.pool)
    .await
    .unwrap();
    assert_eq!(
        shown,
        vec![
            (
                "UPI/DR/512345678901/SHOP".into(),
                Some("512345678901".into()),
                Some("T1".into()),
                Some("11:20:00".into())
            ),
            (
                "UPI/DR/OTHER".into(),
                Some("R7".into()),
                Some("T2".into()),
                Some("09:05:00".into())
            ),
            (
                "NEFT/RENT".into(),
                Some("U3".into()),
                Some("T3".into()),
                Some("10:00:00".into())
            ),
        ]
    );
    db.drop().await;
}
