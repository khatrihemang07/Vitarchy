//! Turning a page address into the site it belongs to.

/// The lowercase host of `address` without a leading `www.`, keeping a non-default port
/// (`localhost:5173`). `None` when the address has no host.
#[must_use]
pub fn domain(address: &str) -> Option<String> {
    let url = url::Url::parse(address).ok()?;
    let host = url.host_str()?;
    let host = host.strip_prefix("www.").unwrap_or(host);
    if host.is_empty() {
        return None;
    }
    // `port()` is already `None` for a scheme's default port.
    Some(
        url.port()
            .map_or_else(|| host.to_owned(), |port| format!("{host}:{port}")),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_www_and_lowercases() {
        assert_eq!(
            domain("https://WWW.Example.com/a?b=1#c").as_deref(),
            Some("example.com")
        );
    }

    #[test]
    fn keeps_other_subdomains_and_non_default_ports() {
        assert_eq!(domain("https://docs.rs/x").as_deref(), Some("docs.rs"));
        assert_eq!(
            domain("http://localhost:5173/app").as_deref(),
            Some("localhost:5173")
        );
        assert_eq!(
            domain("https://example.com:443/").as_deref(),
            Some("example.com")
        );
    }

    #[test]
    fn unparseable_or_hostless_addresses_give_none() {
        for bad in ["", "not a url", "file:///tmp/x", "mailto:a@b.c", "https://"] {
            assert_eq!(domain(bad), None, "{bad:?}");
        }
    }
}
