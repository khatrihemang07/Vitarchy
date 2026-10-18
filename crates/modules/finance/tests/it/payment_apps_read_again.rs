//! A payment history confirmed before the payment-app migration has Printings but no
//! sections; Read again must still work on it, placing each section by its Printings.

use vitarchy_finance::reader::ReaderResult;

use crate::common::{
    app::Env,
    old_shape::{old_database, seed_one_history},
    results::{app_row, history, instrument},
    tx,
};
use vitarchy_finance::reader::testing::FakeReader;

const SHA: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn read_now(second_source_id: &str) -> ReaderResult {
    let shop = app_row(
        tx(1, "Paid to SHOP", 500),
        (11, 20),
        Some("T1"),
        Some("512345678901"),
    );
    let rent = app_row(
        tx(3, "Paid to LANDLORD", 900),
        (10, 0),
        Some(second_source_id),
        Some("U3"),
    );
    history(vec![
        instrument(None, "XXXXXX1111", vec![shop]),
        instrument(None, "ICICI R", vec![rent]),
    ])
}

/// The migrated old-shape database behind the routes, with the history's file stored.
async fn migrated(label: &str, answer: ReaderResult) -> Env {
    let (db, module) = old_database(label).await;
    seed_one_history(&db).await;
    module.migrator().run(&db.pool).await.unwrap();
    let env = Env::on_database(db, FakeReader::with_results([answer]));
    std::fs::write(env.stored_file(SHA), b"synthetic").unwrap();
    env
}

async fn section_accounts(env: &Env) -> Vec<(i32, Option<i64>)> {
    sqlx::query_as("SELECT position, account_id FROM finance.statement_accounts WHERE statement_id = 2 ORDER BY position")
        .fetch_all(&env.db.pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn read_again_on_a_migrated_history_places_each_section_and_changes_nothing() {
    let env = migrated("pa_ra_ok", read_now("T3")).await;
    assert!(
        section_accounts(&env).await.is_empty(),
        "the migration leaves no sections"
    );

    let preview = env.post("/statements/2/read-again", "{}").await;
    assert_eq!(preview.status, 200, "{}", preview.text);
    let applied = env.post("/statements/2/read-again/confirm", "{}").await;
    assert_eq!(applied.status, 200, "{}", applied.text);
    assert_eq!(applied.json["added"], 0, "{}", applied.text);
    assert_eq!(applied.json["printings_added"], 0, "{}", applied.text);
    assert_eq!(applied.json["no_longer_found"], 0, "{}", applied.text);
    assert_eq!(
        section_accounts(&env).await,
        vec![(0, Some(1)), (1, Some(2))]
    );

    let again = env.post("/statements/2/read-again/confirm", "{}").await;
    assert_eq!(again.status, 200, "{}", again.text);
    assert_eq!(again.json["added"], 0);
    env.finish().await;
}

#[tokio::test]
async fn a_section_that_cannot_be_placed_is_refused_with_its_own_code() {
    let env = migrated("pa_ra_unplaced", read_now("T9")).await;
    sqlx::query("DELETE FROM finance.instrument_accounts WHERE instrument = 'ICICI R'")
        .execute(&env.db.pool)
        .await
        .unwrap();

    let reply = env.post("/statements/2/read-again/confirm", "{}").await;

    assert_eq!(reply.status, 409);
    assert_eq!(reply.json["code"], "statement_sections_unplaced");
    assert!(section_accounts(&env).await.is_empty());
    env.finish().await;
}
