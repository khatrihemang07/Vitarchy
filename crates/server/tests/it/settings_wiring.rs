use crate::common::dummy_module;
use vitarchy_core::test_db::TestDb;
use vitarchy_server::{app, migrate, registry};

#[test]
fn registry_lists_settings_as_core_before_modules() {
    let ids: Vec<_> = registry::all()
        .iter()
        .map(vitarchy_core::Module::id)
        .collect();
    assert_eq!(ids.first(), Some(&"settings"));
}

#[test]
fn settings_paths_are_in_the_composed_openapi_document() {
    let spec = serde_json::to_value(app::compose(registry::all()).get_openapi()).unwrap();
    assert!(spec["paths"]["/api/settings"]["get"].is_object(), "{spec}");
    assert!(
        spec["paths"]["/api/settings/core"]["put"].is_object(),
        "{spec}"
    );
    assert!(
        spec["paths"]["/api/settings/modules/{id}"]["put"].is_object(),
        "{spec}"
    );
}

#[tokio::test]
async fn settings_migrations_apply_alongside_another_module() {
    let db = TestDb::create("wiring").await;
    let pool = &db.pool;

    let mut modules = registry::all();
    modules.push(dummy_module());
    migrate::run(pool, &modules).await.unwrap();
    migrate::run(pool, &modules).await.unwrap();

    for table in ["settings.entries", "dummy.item"] {
        let exists: bool = sqlx::query_scalar("SELECT to_regclass($1) IS NOT NULL")
            .bind(table)
            .fetch_one(pool)
            .await
            .unwrap();
        assert!(exists, "{table}");
    }
    db.drop().await;
}
