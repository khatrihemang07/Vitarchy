use vitarchy_server::config::Config;

fn parse(
    db: Option<&str>,
    bind: Option<&str>,
) -> Result<Config, vitarchy_server::config::ConfigError> {
    Config::parse(db.map(str::to_owned), bind.map(str::to_owned))
}

#[test]
fn defaults_to_loopback_port_3000() {
    let c = parse(Some("postgres://x"), None).unwrap();
    assert_eq!(c.bind_addr.to_string(), "127.0.0.1:3000");
}

#[test]
fn rejects_non_loopback_bind_addresses() {
    assert!(parse(Some("postgres://x"), Some("0.0.0.0:3000")).is_err());
    assert!(parse(Some("postgres://x"), Some("192.168.1.5:3000")).is_err());
    assert!(parse(Some("postgres://x"), Some("[::1]:3000")).is_ok());
}

#[test]
fn requires_database_url() {
    assert!(parse(None, None).is_err());
}
