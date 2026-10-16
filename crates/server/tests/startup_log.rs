use vitarchy_core::log_capture::capture_logs;
use vitarchy_server::{config::Config, registry, startup_log};

#[tokio::test]
async fn startup_logs_registered_ids_and_never_the_database_password() {
    let config = Config::parse(
        Some("postgres://vitarchy:hunter2@127.0.0.1:5433/vitarchy".to_owned()),
        Some("127.0.0.1:3104".to_owned()),
    )
    .unwrap();
    let (core, modules) = (registry::core(), registry::modules());

    let ((), logs) = capture_logs(async { startup_log::log(&config, &core, &modules) }).await;

    assert!(!logs.contains("hunter2"), "{logs}");
    assert!(logs.contains("vitarchy:***@127.0.0.1:5433"), "{logs}");
    assert!(logs.contains("127.0.0.1:3104"), "{logs}");
    assert!(logs.contains("settings"), "{logs}");
}
