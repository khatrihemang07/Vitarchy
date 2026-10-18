use vitarchy_core::log_capture::capture_logs;
use vitarchy_server::{
    config::{Config, RawConfig},
    registry, startup_log,
};

#[tokio::test]
async fn startup_logs_registered_ids_and_never_the_database_password() {
    let config = Config::parse(RawConfig {
        database_url: Some("postgres://vitarchy:hunter2@127.0.0.1:5433/vitarchy".to_owned()),
        bind_addr: Some("127.0.0.1:3104".to_owned()),
        web_dist: Some("web/dist".to_owned()),
    })
    .unwrap();
    let (core, modules) = (registry::core(), registry::modules());

    let ((), logs) = capture_logs(async { startup_log::log(&config, &core, &modules) }).await;

    assert!(!logs.contains("hunter2"), "{logs}");
    assert!(logs.contains("vitarchy:***@127.0.0.1:5433"), "{logs}");
    assert!(logs.contains("127.0.0.1:3104"), "{logs}");
    assert!(logs.contains("settings"), "{logs}");
}

#[tokio::test]
async fn startup_logs_which_mode_and_dist_dir() {
    let parse = |dist: Option<&str>| {
        Config::parse(RawConfig {
            database_url: Some("postgres://u@127.0.0.1:5434/d".to_owned()),
            bind_addr: None,
            web_dist: dist.map(str::to_owned),
        })
        .unwrap()
    };
    let (core, modules) = (registry::core(), registry::modules());

    let ((), serving) =
        capture_logs(async { startup_log::log(&parse(Some("web/dist")), &core, &modules) }).await;
    let ((), api_only) =
        capture_logs(async { startup_log::log(&parse(None), &core, &modules) }).await;

    assert!(
        serving.contains("serves web build") && serving.contains("web/dist"),
        "{serving}"
    );
    assert!(api_only.contains("api only"), "{api_only}");
}
