use vitarchy_server::config::{Config, ConfigError, RawConfig};

fn parse(
    db: Option<&str>,
    bind: Option<&str>,
    web_dist: Option<&str>,
) -> Result<Config, ConfigError> {
    Config::parse(RawConfig {
        database_url: db.map(str::to_owned),
        bind_addr: bind.map(str::to_owned),
        web_dist: web_dist.map(str::to_owned),
    })
}

#[test]
fn defaults_to_loopback_port_3000() {
    let c = parse(Some("postgres://x"), None, None).unwrap();
    assert_eq!(c.bind_addr.to_string(), "127.0.0.1:3000");
}

#[test]
fn rejects_non_loopback_bind_addresses() {
    assert!(parse(Some("postgres://x"), Some("0.0.0.0:3000"), None).is_err());
    assert!(parse(Some("postgres://x"), Some("192.168.1.5:3000"), None).is_err());
    assert!(parse(Some("postgres://x"), Some("[::1]:3000"), None).is_ok());
}

#[test]
fn requires_database_url() {
    assert!(parse(None, None, None).is_err());
}

#[test]
fn web_dist_is_optional_and_empty_means_unset() {
    assert!(
        parse(Some("postgres://x"), None, None)
            .unwrap()
            .web_dist
            .is_none()
    );
    assert!(
        parse(Some("postgres://x"), None, Some(""))
            .unwrap()
            .web_dist
            .is_none()
    );
    let c = parse(Some("postgres://x"), None, Some("web/dist")).unwrap();
    assert_eq!(c.web_dist.unwrap().to_str(), Some("web/dist"));
}
