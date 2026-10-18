use crate::common::finance_db;
use vitarchy_finance::repo::passwords;

#[tokio::test]
async fn secrets_come_back_oldest_first_and_labels_never_hold_them() {
    let db = finance_db("pw_order").await;
    let first = passwords::insert(&db.pool, "ICICI", "Alpha123")
        .await
        .unwrap();
    passwords::insert(&db.pool, "SBI", "Beta456").await.unwrap();
    let secrets = passwords::secrets(&db.pool).await.unwrap();
    assert_eq!(
        secrets
            .iter()
            .map(|p| p.secret.as_str())
            .collect::<Vec<_>>(),
        ["Alpha123", "Beta456"]
    );
    assert_eq!(secrets[0].id, first);
    let labels = format!("{:?}", passwords::labels(&db.pool).await.unwrap());
    assert!(
        labels.contains("ICICI") && !labels.contains("Alpha123"),
        "{labels}"
    );
    db.drop().await;
}

#[tokio::test]
async fn debug_output_hides_the_secret() {
    let db = finance_db("pw_debug").await;
    passwords::insert(&db.pool, "x", "Sup3rSecret")
        .await
        .unwrap();
    let shown = format!("{:?}", passwords::secrets(&db.pool).await.unwrap());
    assert!(!shown.contains("Sup3rSecret"), "{shown}");
    db.drop().await;
}

#[tokio::test]
async fn touch_stamps_last_used_and_delete_forgets() {
    let db = finance_db("pw_touch").await;
    let id = passwords::insert(&db.pool, "x", "s").await.unwrap();
    assert!(
        passwords::labels(&db.pool).await.unwrap()[0]
            .last_used_at
            .is_none()
    );
    passwords::touch(&db.pool, id).await.unwrap();
    assert!(
        passwords::labels(&db.pool).await.unwrap()[0]
            .last_used_at
            .is_some()
    );
    assert!(passwords::delete(&db.pool, id).await.unwrap());
    assert!(!passwords::delete(&db.pool, id).await.unwrap());
    db.drop().await;
}
