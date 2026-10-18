//! Repairing duplicate Accounts: they merge into the oldest, overlapping rows become Printings
//! of one Transaction, and nothing is lost.

use crate::common::{
    account,
    app::Env,
    cards::{MASK, card},
    results::{read, section},
    tx,
};
use serde_json::json;
use vitarchy_finance::{
    merge_duplicate_accounts,
    reader::contract::{AccountKind, Transaction},
    repo::{accounts, instrument_accounts},
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
async fn overlapping_identical_rows_fold_into_one_transaction_with_one_printing() {
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
        (4, 4),
        "identical Printings are not moved onto the kept Transaction"
    );
    assert_eq!(report.printings, 0);
    env.finish().await;
}

#[tokio::test]
async fn a_folded_row_that_prints_differently_still_moves_its_printing() {
    let files = [vec![tx(3, "bravo", 200)], vec![tx(3, "bravo", 200)]];
    let mut richer = files[1].clone();
    richer[0].reference = Some("R-1".into());
    let env = Env::new("mg_differs", vec![card(files[0].clone()), card(richer)]).await;
    for name in ["a", "b"] {
        let id = account(&env.db, MASK).await.id;
        let statement = env
            .upload_one(&format!("{name}.pdf"), name.as_bytes())
            .await;
        let body = json!({"sections": [{"account_id": id}]});
        let path = format!("/statements/{statement}/confirm");
        assert_eq!(env.post(&path, &body.to_string()).await.status, 200);
    }

    let report = merge_duplicate_accounts(&env.db.pool).await.unwrap();

    assert_eq!((report.folded, report.printings), (1, 1));
    assert_eq!(count(&env, "printings").await, 2);
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
async fn a_remembered_instrument_follows_its_account_through_a_merge() {
    let env = Env::new("mg_instrument", vec![savings(vec![tx(2, "other", 100)])]).await;
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
    }
    instrument_accounts::remember(&env.db.pool, "XXXXXX4321", ids[1])
        .await
        .unwrap();

    merge_duplicate_accounts(&env.db.pool).await.unwrap();

    let kept = instrument_accounts::all(&env.db.pool).await.unwrap();
    assert_eq!(kept.get("XXXXXX4321"), Some(&ids[0]));
    env.finish().await;
}
