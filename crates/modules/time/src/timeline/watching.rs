//! Where the person is watching or in a call, so no input there is not leaving (data only).

use super::domain::domain;

/// Apps whose no-input time stays Active up to the watching cap: players and call apps.
/// Spotify is deliberately absent: listening does not need the screen.
pub const WATCHING_APPS: [&str; 6] = [
    "VLC",
    "IINA",
    "QuickTime Player",
    "zoom.us",
    "FaceTime",
    "Microsoft Teams",
];

/// Sites (and their subdomains) with the same exception.
pub const WATCHING_SITES: [&str; 5] = [
    "youtube.com",
    "netflix.com",
    "meet.google.com",
    "primevideo.com",
    "hotstar.com",
];

/// Whether `app`, or the site of `url`, is somewhere people watch without touching the keys.
#[must_use]
pub fn is_watching(app: &str, url: Option<&str>) -> bool {
    WATCHING_APPS.iter().any(|a| a.eq_ignore_ascii_case(app))
        || url
            .and_then(domain)
            .is_some_and(|host| on_a_watching_site(&host))
}

/// The host is a watching site or one of its subdomains. A port is ignored.
fn on_a_watching_site(host: &str) -> bool {
    let host = host.split(':').next().unwrap_or(host);
    WATCHING_SITES
        .iter()
        .any(|site| host == *site || host.ends_with(&format!(".{site}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn players_and_call_apps_are_watching() {
        for app in ["VLC", "zoom.us", "Microsoft Teams", "quicktime player"] {
            assert!(is_watching(app, None), "{app}");
        }
    }

    #[test]
    fn spotify_is_not_watching() {
        assert!(!is_watching("Spotify", None));
    }

    #[test]
    fn a_watching_site_matches_with_www_and_subdomains() {
        for url in [
            "https://youtube.com/watch?v=1",
            "https://www.youtube.com/watch?v=1",
            "https://m.youtube.com/watch?v=1",
            "https://meet.google.com/abc",
        ] {
            assert!(is_watching("Brave", Some(url)), "{url}");
        }
    }

    #[test]
    fn other_sites_and_lookalikes_are_not_watching() {
        for url in [
            "https://google.com",
            "https://notyoutube.com",
            "https://youtube.com.evil.io",
        ] {
            assert!(!is_watching("Brave", Some(url)), "{url}");
        }
        assert!(!is_watching("Brave", None));
    }
}
