use crate::common::{account, finance_db, payment, statement, tx};
use vitarchy_finance::{
    RecordFacts, TransactionFacts, link,
    reader::contract::Direction,
    repo::{payment_records, transactions},
};

#[tokio::test]
async fn a_repeated_source_id_is_not_stored_twice() {
    let db = finance_db("pr_repeat").await;
    let history = statement(&db, 'e').await;
    assert!(
        payment_records::insert(&db.pool, history, &payment(Some("T1"), None, 100))
            .await
            .unwrap()
    );
    assert!(
        !payment_records::insert(&db.pool, history, &payment(Some("T1"), None, 100))
            .await
            .unwrap()
    );
    assert!(
        payment_records::insert(&db.pool, history, &payment(None, None, 100))
            .await
            .unwrap()
    );
    assert!(
        payment_records::insert(&db.pool, history, &payment(None, None, 100))
            .await
            .unwrap()
    );
    assert_eq!(payment_records::unlinked(&db.pool).await.unwrap().len(), 3);
    db.drop().await;
}

#[tokio::test]
async fn linking_by_utr_through_the_repos_pairs_a_record_with_its_transaction() {
    let db = finance_db("pr_link").await;
    let account = account(&db, "XXXXXX4321").await;
    let bank = statement(&db, 'a').await;
    let history = statement(&db, 'e').await;
    let mut row = tx(5, "UPI/DR/512345678901/SHOP", 100);
    row.reference = Some("512345678901".into());
    let tx_id = transactions::insert(&db.pool, account.id, bank, &row)
        .await
        .unwrap();
    payment_records::insert(
        &db.pool,
        history,
        &payment(Some("T1"), Some("512345678901"), 100),
    )
    .await
    .unwrap();

    let records: Vec<RecordFacts> = payment_records::unlinked(&db.pool)
        .await
        .unwrap()
        .into_iter()
        .map(|r| RecordFacts {
            id: r.id,
            on: r.occurred_at.date(),
            amount_paise: r.amount_paise,
            direction: Direction::Debit,
            utr: r.utr,
            instrument_issuer: r.instrument_issuer,
            instrument_digits: r.instrument_digits,
        })
        .collect();
    let candidates: Vec<TransactionFacts> = transactions::without_payment_record(&db.pool)
        .await
        .unwrap()
        .into_iter()
        .map(|t| TransactionFacts {
            id: t.row.id,
            date: t.row.date,
            amount_paise: t.row.amount_paise,
            direction: Direction::Debit,
            reference: t.row.reference,
            description: t.row.description,
            account_issuer: t.account_issuer,
            account_mask: t.account_mask,
        })
        .collect();
    let links = link(&records, &candidates);
    assert_eq!(links, vec![(records[0].id, tx_id)]);
    for (record, transaction) in links {
        assert!(
            payment_records::link(&db.pool, record, transaction)
                .await
                .unwrap()
        );
    }
    assert!(
        payment_records::unlinked(&db.pool)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        transactions::without_payment_record(&db.pool)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        payment_records::for_transaction(&db.pool, tx_id)
            .await
            .unwrap()
            .is_some()
    );
    db.drop().await;
}

#[tokio::test]
async fn a_transaction_cannot_carry_two_records() {
    let db = finance_db("pr_one").await;
    let account = account(&db, "XX2003").await;
    let bank = statement(&db, 'a').await;
    let history = statement(&db, 'e').await;
    let tx_id = transactions::insert(&db.pool, account.id, bank, &tx(5, "x", 100))
        .await
        .unwrap();
    payment_records::insert(&db.pool, history, &payment(Some("A"), None, 100))
        .await
        .unwrap();
    payment_records::insert(&db.pool, history, &payment(Some("B"), None, 100))
        .await
        .unwrap();
    let ids: Vec<i64> = payment_records::unlinked(&db.pool)
        .await
        .unwrap()
        .iter()
        .map(|r| r.id)
        .collect();
    payment_records::link(&db.pool, ids[0], tx_id)
        .await
        .unwrap();
    assert!(
        payment_records::link(&db.pool, ids[1], tx_id)
            .await
            .is_err()
    );
    db.drop().await;
}
