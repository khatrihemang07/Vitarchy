use crate::common::finance_db;
use vitarchy_finance::{
    Match, match_account,
    reader::contract::AccountKind,
    repo::{account_identifiers, accounts},
};

#[tokio::test]
async fn an_account_stores_the_last_four_digits_of_its_mask() {
    let db = finance_db("acc_last4").await;
    let account = crate::common::account(&db, "4123XXXXXXXX2003").await;
    assert_eq!(account.last4.as_deref(), Some("2003"));
    let none = accounts::insert(&db.pool, "SBI", AccountKind::Loan, None, "Loan")
        .await
        .unwrap();
    assert_eq!(none.last4, None);
    db.drop().await;
}

#[tokio::test]
async fn known_lists_accounts_and_remembered_identifiers_for_matching() {
    let db = finance_db("acc_known").await;
    let account = crate::common::account(&db, "XX2003").await;
    accounts::insert(&db.pool, "SBI", AccountKind::Loan, None, "No digits")
        .await
        .unwrap();
    account_identifiers::remember(
        &db.pool,
        account.id,
        "ICICI Bank",
        AccountKind::CreditCard,
        "2222",
    )
    .await
    .unwrap();
    account_identifiers::remember(
        &db.pool,
        account.id,
        "ICICI Bank",
        AccountKind::CreditCard,
        "2222",
    )
    .await
    .unwrap();

    let known = accounts::identities(&db.pool).await.unwrap();
    assert_eq!(known.len(), 3, "{known:?}");
    let by_identifier = match_account(
        Some("ICICI Bank"),
        Some(AccountKind::CreditCard),
        Some("XXXX2222"),
        &known,
    );
    assert_eq!(by_identifier, Match::One(account.id));
    db.drop().await;
}

#[tokio::test]
async fn accounts_can_be_listed_fetched_and_renamed() {
    let db = finance_db("acc_rename").await;
    let account = crate::common::account(&db, "XX2003").await;
    assert!(
        accounts::rename(&db.pool, account.id, "Travel card")
            .await
            .unwrap()
    );
    assert!(!accounts::rename(&db.pool, 9999, "x").await.unwrap());
    assert_eq!(
        accounts::get(&db.pool, account.id)
            .await
            .unwrap()
            .unwrap()
            .name,
        "Travel card"
    );
    assert_eq!(accounts::list(&db.pool).await.unwrap().len(), 1);
    db.drop().await;
}
