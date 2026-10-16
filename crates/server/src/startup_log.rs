//! What the server logs once at startup: effective config (secrets removed)
//! and what is registered.

use url::Url;
use vitarchy_core::Module;

use crate::config::Config;

/// Shown instead of a URL we could not safely redact.
const UNPARSEABLE: &str = "<unparseable url>";

/// `DATABASE_URL` safe to log: the password is masked and the query string
/// and fragment (which can carry `?password=`) are dropped. Anything that does
/// not parse as a URL is replaced wholesale rather than risk leaking it. As
/// for sqlx, special characters in a password must be percent-encoded.
#[must_use]
pub fn redact_url(raw: &str) -> String {
    let Ok(mut url) = Url::parse(raw) else {
        return UNPARSEABLE.to_owned();
    };
    url.set_query(None);
    url.set_fragment(None);
    if url.password().is_some() {
        // Setting a password only fails for URLs that cannot hold one, and this one has one.
        url.set_password(Some("***")).ok();
    }
    url.into()
}

/// Logs the effective configuration and the ids of the registered Core
/// features and Modules.
pub fn log(config: &Config, core: &[Module], modules: &[Module]) {
    let ids = |list: &[Module]| list.iter().map(Module::id).collect::<Vec<_>>();
    tracing::info!(
        database_url = %redact_url(&config.database_url),
        bind_addr = %config.bind_addr,
        "configuration"
    );
    tracing::info!(core = ?ids(core), modules = ?ids(modules), "registered");
}

#[cfg(test)]
mod tests {
    use super::redact_url;

    #[test]
    fn masks_the_password() {
        assert_eq!(
            redact_url("postgres://vitarchy:s3cret@127.0.0.1:5433/vitarchy"),
            "postgres://vitarchy:***@127.0.0.1:5433/vitarchy"
        );
    }

    #[test]
    fn keeps_urls_without_a_password() {
        assert_eq!(
            redact_url("postgres://vitarchy@localhost/db"),
            "postgres://vitarchy@localhost/db"
        );
        assert_eq!(
            redact_url("postgres://localhost/db"),
            "postgres://localhost/db"
        );
    }

    #[test]
    fn password_containing_at_sign_is_fully_masked() {
        assert_eq!(
            redact_url("postgres://u:p@ss@host/db"),
            "postgres://u:***@host/db"
        );
    }

    #[test]
    fn drops_the_query_string() {
        assert_eq!(
            redact_url("postgres://u@host/db?password=hunter2&sslmode=disable"),
            "postgres://u@host/db"
        );
        assert_eq!(
            redact_url("postgres://u@host?password=x"),
            "postgres://u@host"
        );
    }

    #[test]
    fn percent_encoded_slash_question_and_hash_in_a_password_are_masked() {
        assert_eq!(
            redact_url("postgres://u:p%2Fa%3Fb%23c@host:5433/db"),
            "postgres://u:***@host:5433/db"
        );
    }

    #[test]
    fn drops_the_fragment() {
        assert_eq!(
            redact_url("postgres://u@host/db#secret"),
            "postgres://u@host/db"
        );
    }

    #[test]
    fn non_urls_are_not_echoed() {
        assert_eq!(redact_url("hunter2"), "<unparseable url>");
    }
}
