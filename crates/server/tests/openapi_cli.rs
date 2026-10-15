use std::process::Command;

#[test]
fn openapi_subcommand_prints_spec_without_a_database() {
    let out = Command::new(env!("CARGO_BIN_EXE_vitarchy-server"))
        .arg("openapi")
        .env_remove("DATABASE_URL")
        .env_remove("BIND_ADDR")
        .current_dir(std::env::temp_dir()) // no .env to pick up
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let spec: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(spec["openapi"].as_str().unwrap().starts_with("3."));
    assert!(spec["paths"]["/api/health"]["get"].is_object());
}

#[test]
fn unknown_subcommand_fails_with_usage() {
    let out = Command::new(env!("CARGO_BIN_EXE_vitarchy-server"))
        .arg("bogus")
        .output()
        .unwrap();

    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("usage"));
}
