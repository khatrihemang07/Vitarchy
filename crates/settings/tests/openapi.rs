use utoipa_axum::router::OpenApiRouter;

#[test]
fn openapi_documents_the_three_settings_paths() {
    let router = OpenApiRouter::new().nest("/api/settings", vitarchy_settings::feature().router());
    let spec = serde_json::to_value(router.get_openapi()).unwrap();

    assert!(spec["paths"]["/api/settings"]["get"].is_object(), "{spec}");
    assert!(
        spec["paths"]["/api/settings/core"]["put"].is_object(),
        "{spec}"
    );
    assert!(
        spec["paths"]["/api/settings/modules/{id}"]["put"].is_object(),
        "{spec}"
    );
    assert!(
        spec["components"]["schemas"]["CoreSettings"].is_object(),
        "{spec}"
    );
}
