use crate::common::{finance_db, statement};
use vitarchy_finance::repo::{
    statement_accounts::{self, StatementAccountRow},
    statements,
};

#[tokio::test]
async fn a_new_statement_is_unconfirmed_until_confirmed_once() {
    let db = finance_db("st_confirm").await;
    let id = statement(&db, 'a').await;
    let row = statements::get(&db.pool, id).await.unwrap().unwrap();
    assert_eq!(
        (row.status.as_str(), row.confirmed_at.is_none()),
        ("unconfirmed", true)
    );
    assert!(statements::confirm(&db.pool, id).await.unwrap());
    assert!(!statements::confirm(&db.pool, id).await.unwrap());
    assert!(
        statements::get(&db.pool, id)
            .await
            .unwrap()
            .unwrap()
            .confirmed_at
            .is_some()
    );
    db.drop().await;
}

#[tokio::test]
async fn only_an_unconfirmed_statement_can_be_discarded() {
    let db = finance_db("st_discard").await;
    let open = statement(&db, 'a').await;
    let done = statement(&db, 'b').await;
    statements::confirm(&db.pool, done).await.unwrap();
    assert!(statements::discard(&db.pool, open).await.unwrap());
    assert!(!statements::discard(&db.pool, done).await.unwrap());
    assert_eq!(statements::list(&db.pool).await.unwrap().len(), 1);
    db.drop().await;
}

#[tokio::test]
async fn file_in_use_tells_whether_another_statement_shares_the_original() {
    let db = finance_db("st_inuse").await;
    let id = statement(&db, 'c').await;
    let sha = "c".repeat(64);
    assert!(statements::file_in_use(&db.pool, &sha).await.unwrap());
    statements::discard(&db.pool, id).await.unwrap();
    assert!(!statements::file_in_use(&db.pool, &sha).await.unwrap());
    db.drop().await;
}

#[tokio::test]
async fn statement_accounts_keep_printed_order_and_take_an_assignment() {
    let db = finance_db("st_accounts").await;
    let id = statement(&db, 'a').await;
    let account = crate::common::account(&db, "XX2003").await;
    for position in [1, 0] {
        let row = StatementAccountRow {
            position,
            account_id: None,
            proposed_issuer: Some("ICICI Bank".into()),
            proposed_kind: Some("credit_card".into()),
            proposed_mask: Some("XX2003".into()),
        };
        statement_accounts::insert(&db.pool, id, &row)
            .await
            .unwrap();
    }
    assert!(
        statement_accounts::assign(&db.pool, id, 1, account.id)
            .await
            .unwrap()
    );
    let rows = statement_accounts::list(&db.pool, id).await.unwrap();
    assert_eq!(
        rows.iter()
            .map(|r| (r.position, r.account_id))
            .collect::<Vec<_>>(),
        [(0, None), (1, Some(account.id))]
    );
    statement_accounts::clear(&db.pool, id).await.unwrap();
    assert!(
        statement_accounts::list(&db.pool, id)
            .await
            .unwrap()
            .is_empty()
    );
    db.drop().await;
}
