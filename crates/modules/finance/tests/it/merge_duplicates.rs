//! Repairing duplicate Accounts: they merge into the oldest, overlapping rows become Printings
//! of one Transaction, and nothing is lost.

use crate::common::{
    account,
    app::Env,
    cards::{MASK, card},
    payment,
    results::{history, read, section},
    tx,
};
use serde_json::json;
use vitarchy_finance::{
    merge_duplicate_accounts,
    reader::contract::{AccountKind, Transaction},
    repo::accounts,
};

async fn count(env: &Env, table: &str) -> i64 {
    sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "SELECT count(*) FROM finance.{table}"
    )))
    .fetch_one(&env.db.pool)
    .await
    .unwrap()
}

/// Three Statements with overlapping rows, each confirmed to its own Account of one card
/// (what the page-style confirms did before the fix). Returns the Accounts, oldest first.
async fn three_duplicates(label: &str) -> (Env, Vec<i64>) {
    let files = vec![
        vec![tx(2, "alpha", 100), tx(3, "bravo", 200)],
        vec![tx(3, "bravo", 200), tx(4, "charlie", 300)],
        vec![tx(4, "charlie", 300), tx(5, "delta", 400)],
    ];
    let env = Env::new(label, files.into_iter().map(card).collect()).await;
    let mut ids = Vec::new();
    for name in ["a", "b", "c"] {
        ids.push(account(&env.db, MASK).await.id);
        let statement = env
            .upload_one(&format!("{name}.pdf"), name.as_bytes())
            .await;
        let body = json!({"sections": [{"account_id": ids.last()}]});
        let reply = env
            .post(
                &format!("/statements/{statement}/confirm"),
                &body.to_string(),
            )
            .await;
        assert_eq!(reply.status, 200, "{}", reply.text);
    }
    (env, ids)
}

#[tokio::test]
async fn three_duplicate_accounts_merge_into_the_oldest() {
    let (env, ids) = three_duplicates("mg_three").await;

    let report = merge_duplicate_accounts(&env.db.pool).await.unwrap();

    assert_eq!((report.groups, report.accounts_merged), (1, 2));
    let left = accounts::list(&env.db.pool).await.unwrap();
    assert_eq!(left.iter().map(|a| a.id).collect::<Vec<_>>(), vec![ids[0]]);
    let pointing: i64 =
        sqlx::query_scalar("SELECT count(*) FROM finance.statement_accounts WHERE account_id = $1")
            .bind(ids[0])
            .fetch_one(&env.db.pool)
            .await
            .unwrap();
    assert_eq!(pointing, 3);
    env.finish().await;
}

#[tokio::test]
async fn overlapping_rows_become_printings_of_one_transaction_and_nothing_is_lost() {
    let (env, _) = three_duplicates("mg_overlap").await;
    assert_eq!(
        (
            count(&env, "transactions").await,
            count(&env, "printings").await
        ),
        (6, 6)
    );

    let report = merge_duplicate_accounts(&env.db.pool).await.unwrap();

    assert_eq!((report.moved, report.folded), (2, 2));
    assert_eq!(
        (
            count(&env, "transactions").await,
            count(&env, "printings").await
        ),
        (4, 6)
    );
    let shared: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM (SELECT transaction_id FROM finance.printings
         GROUP BY transaction_id HAVING count(*) = 2) two",
    )
    .fetch_one(&env.db.pool)
    .await
    .unwrap();
    assert_eq!(shared, 2);
    env.finish().await;
}

#[tokio::test]
async fn a_second_run_changes_nothing() {
    let (env, _) = three_duplicates("mg_again").await;
    merge_duplicate_accounts(&env.db.pool).await.unwrap();
    let before = (
        count(&env, "accounts").await,
        count(&env, "transactions").await,
        count(&env, "printings").await,
    );

    let report = merge_duplicate_accounts(&env.db.pool).await.unwrap();

    assert_eq!((report.groups, report.accounts_merged), (0, 0));
    assert_eq!(
        before,
        (
            count(&env, "accounts").await,
            count(&env, "transactions").await,
            count(&env, "printings").await
        )
    );
    env.finish().await;
}

#[tokio::test]
async fn accounts_that_differ_in_number_or_kind_stay_apart() {
    let env = Env::new("mg_apart", vec![card(vec![tx(2, "A", 100)])]).await;
    account(&env.db, "XXXX2003").await;
    account(&env.db, "XXXX9999").await;
    accounts::insert(
        &env.db.pool,
        "ICICI Bank",
        AccountKind::Savings,
        Some("XXXX2003"),
        "Other",
    )
    .await
    .unwrap();
    accounts::insert(
        &env.db.pool,
        "ICICI Bank",
        AccountKind::Savings,
        None,
        "No number",
    )
    .await
    .unwrap();

    let report = merge_duplicate_accounts(&env.db.pool).await.unwrap();

    assert_eq!(report.accounts_merged, 0);
    assert_eq!(count(&env, "accounts").await, 4);
    env.finish().await;
}

fn savings(rows: Vec<Transaction>) -> vitarchy_finance::reader::ReaderResult {
    read(vec![section(
        Some("SBI"),
        Some(AccountKind::Savings),
        "XXXXXX4321",
        rows,
    )])
}

#[tokio::test]
async fn payment_record_links_survive_a_merge() {
    let mut paid = tx(5, "UPI/DR/512345678901/SOME MERCHANT", 25_000);
    paid.reference = Some("512345678901".to_owned());
    let env = Env::new(
        "mg_links",
        vec![
            savings(vec![tx(2, "other", 100)]),
            savings(vec![paid.clone()]),
            history(vec![payment(Some("R1"), Some("512345678901"), 25_000)]),
        ],
    )
    .await;
    let mut ids = Vec::new();
    for name in ["a", "b"] {
        let made = accounts::insert(
            &env.db.pool,
            "SBI",
            AccountKind::Savings,
            Some("XXXXXX4321"),
            name,
        )
        .await
        .unwrap();
        ids.push(made.id);
        let statement = env
            .upload_one(&format!("{name}.pdf"), name.as_bytes())
            .await;
        let body = json!({"sections": [{"account_id": made.id}]});
        env.post(
            &format!("/statements/{statement}/confirm"),
            &body.to_string(),
        )
        .await;
    }
    let phone = env.upload_one("phone.csv", b"history").await;
    env.confirm_proposed(phone, 0).await;
    let linked_before = count_linked(&env).await;

    merge_duplicate_accounts(&env.db.pool).await.unwrap();

    assert_eq!((linked_before, count_linked(&env).await), (1, 1));
    let account_of_linked: i64 = sqlx::query_scalar(
        "SELECT t.account_id FROM finance.payment_records p
         JOIN finance.transactions t ON t.id = p.transaction_id",
    )
    .fetch_one(&env.db.pool)
    .await
    .unwrap();
    assert_eq!(account_of_linked, ids[0]);
    env.finish().await;
}

async fn count_linked(env: &Env) -> i64 {
    sqlx::query_scalar(
        "SELECT count(*) FROM finance.payment_records WHERE transaction_id IS NOT NULL",
    )
    .fetch_one(&env.db.pool)
    .await
    .unwrap()
}

#[tokio::test]
async fn a_record_linked_to_a_folded_transaction_moves_to_the_kept_one() {
    let mut paid = tx(5, "UPI/DR/512345678901/SOME MERCHANT", 25_000);
    paid.reference = Some("512345678901".to_owned());
    let env = Env::new(
        "mg_fold_link",
        vec![
            savings(vec![paid.clone()]),
            savings(vec![paid]),
            history(vec![payment(Some("R1"), Some("512345678901"), 25_000)]),
        ],
    )
    .await;
    for name in ["a", "b"] {
        let made = accounts::insert(
            &env.db.pool,
            "SBI",
            AccountKind::Savings,
            Some("XXXXXX4321"),
            name,
        )
        .await
        .unwrap();
        let statement = env
            .upload_one(&format!("{name}.pdf"), name.as_bytes())
            .await;
        let body = json!({"sections": [{"account_id": made.id}]});
        env.post(
            &format!("/statements/{statement}/confirm"),
            &body.to_string(),
        )
        .await;
    }
    let phone = env.upload_one("phone.csv", b"history").await;
    env.confirm_proposed(phone, 0).await;
    // The record links to the first free Transaction; make it the later one to force a move.
    sqlx::query("UPDATE finance.payment_records SET transaction_id = (SELECT max(id) FROM finance.transactions)")
        .execute(&env.db.pool)
        .await
        .unwrap();

    merge_duplicate_accounts(&env.db.pool).await.unwrap();

    assert_eq!(
        (count(&env, "transactions").await, count_linked(&env).await),
        (1, 1)
    );
    env.finish().await;
}
