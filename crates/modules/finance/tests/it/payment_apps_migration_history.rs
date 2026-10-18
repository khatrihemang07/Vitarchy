//! The migration that retires Payment records: histories with unlinked ones are read again,
//! and instruments are remembered where unambiguous.

use crate::common::old_shape::{old_database, seed_ambiguous_instrument, seed_two_histories};

#[tokio::test]
async fn a_history_with_an_unlinked_record_goes_back_to_unconfirmed_to_be_read_again() {
    let (db, module) = old_database("pa_mig_reopen").await;
    seed_two_histories(&db).await;

    module.migrator().run(&db.pool).await.unwrap();

    let states: Vec<(String, String, String, bool, bool)> = sqlx::query_as(
        "SELECT file_name, status, outcome, confirmed_at IS NULL, result IS NULL
         FROM finance.statements ORDER BY id",
    )
    .fetch_all(&db.pool)
    .await
    .unwrap();
    assert_eq!(
        states,
        vec![
            (
                "bank.pdf".into(),
                "confirmed".into(),
                "read".into(),
                false,
                false
            ),
            (
                "fully.csv".into(),
                "confirmed".into(),
                "read".into(),
                false,
                false
            ),
            (
                "partly.csv".into(),
                "unconfirmed".into(),
                "crashed".into(),
                true,
                true
            ),
        ],
        "the file stays; only its stale result is cleared so Read again fills it"
    );
    let table: Option<String> =
        sqlx::query_scalar("SELECT to_regclass('finance.payment_records')::text")
            .fetch_one(&db.pool)
            .await
            .unwrap();
    assert_eq!(table, None, "payment records are gone");
    db.drop().await;
}

#[tokio::test]
async fn an_instrument_is_remembered_only_when_its_records_landed_on_one_account() {
    let (db, module) = old_database("pa_mig_instruments").await;
    seed_two_histories(&db).await;
    seed_ambiguous_instrument(&db).await;

    module.migrator().run(&db.pool).await.unwrap();

    let remembered: Vec<(String, i64)> = sqlx::query_as(
        "SELECT instrument, account_id FROM finance.instrument_accounts ORDER BY instrument",
    )
    .fetch_all(&db.pool)
    .await
    .unwrap();
    assert_eq!(
        remembered,
        vec![("ICICI R".into(), 2), ("XXXXXX1111".into(), 1)],
        "spacing and case are folded; XXXXXX9999 landed on two Accounts and is left to the person"
    );
    db.drop().await;
}
