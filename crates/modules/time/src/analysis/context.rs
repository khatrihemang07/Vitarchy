//! Derives a Context (GLOSSARY) from an app and a window title: the code
//! workspace for an editor, the site for a browser, the app itself otherwise.
//! Pure; the rules are data in `context_rules.rs`.

use super::context_rules::{APP_RULES, AUDIO_MARKS, EDITOR_SEPARATOR, Matcher, OTHER_PAGES, SITES};

/// Marks that can sit invisibly around an app name (direction marks, zero-width characters).
fn is_invisible(c: char) -> bool {
    c.is_whitespace()
        || matches!(c, '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2060}'..='\u{2069}' | '\u{FEFF}')
}

/// The app name without surrounding whitespace or invisible marks.
#[must_use]
pub fn clean_app(raw: &str) -> String {
    raw.trim_matches(is_invisible).to_owned()
}

/// The Context of `title` seen in `app` (an already cleaned app name). A title
/// that names nothing, or an app with no rule, gives the app's own name.
#[must_use]
pub fn derive_context(app: &str, title: &str) -> String {
    let title = title.trim_matches(is_invisible);
    if title.is_empty() {
        return app.to_owned();
    }
    APP_RULES
        .iter()
        .find(|rule| rule.apps.iter().any(|name| name.eq_ignore_ascii_case(app)))
        .map_or_else(|| app.to_owned(), |rule| (rule.extract)(title))
}

/// An editor's workspace: the text after the last separator, else the whole title.
pub(super) fn editor_workspace(title: &str) -> String {
    let workspace = title
        .rsplit_once(EDITOR_SEPARATOR)
        .map_or(title, |(_, after)| after.trim());
    if workspace.is_empty() {
        title
    } else {
        workspace
    }
    .to_owned()
}

/// A browser page's site: the first known site the title names, else "Other pages".
pub(super) fn browser_site(title: &str) -> String {
    // Browsers append a speaker mark to the title of a tab that is playing or muted.
    let title = title.trim_end_matches(AUDIO_MARKS);
    let found = SITES
        .iter()
        .find(|site| site.matchers.iter().any(|matcher| matches(matcher, title)))
        .map_or(OTHER_PAGES, |site| site.name);
    found.to_owned()
}

fn matches(matcher: &Matcher, title: &str) -> bool {
    match matcher {
        Matcher::Suffix(suffix) => {
            let (title, suffix) = (title.to_lowercase(), suffix.to_lowercase());
            title.ends_with(&suffix)
        }
        Matcher::Exact(text) => title.eq_ignore_ascii_case(text),
        Matcher::RepoPath => is_repo_path(title.split(':').next().unwrap_or(title)),
    }
}

/// `owner/repo`: one slash between two non-empty names of letters, digits, `-`, `_` or `.`.
fn is_repo_path(text: &str) -> bool {
    let name = |part: &str| {
        !part.is_empty()
            && part
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    };
    text.split_once('/')
        .is_some_and(|(owner, repo)| name(owner) && name(repo))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(app: &str, title: &str) -> String {
        derive_context(&clean_app(app), title)
    }

    #[test]
    fn an_editor_title_gives_the_text_after_the_last_separator() {
        assert_eq!(context("Code", "README.md — meologue_01"), "meologue_01");
        assert_eq!(
            context("Cursor", "src/main.rs — notes — LifeTracker_02"),
            "LifeTracker_02"
        );
        assert_eq!(
            context("Code", "Welcome — LifeTracker_02"),
            "LifeTracker_02"
        );
    }

    #[test]
    fn an_editor_title_without_a_separator_is_the_workspace_itself() {
        assert_eq!(context("Code", "meologue_01"), "meologue_01");
    }

    #[test]
    fn a_tab_of_the_persons_own_site_or_a_search_assistant_is_named() {
        assert_eq!(context("Brave Browser", "meologue"), "meologue");
        assert_eq!(
            context("Brave Browser", "Plan a trip - Google Gemini"),
            "Gemini"
        );
        assert_eq!(context("Brave Browser", "Arrays | TUF+"), "takeUforward");
    }

    #[test]
    fn a_hyphen_is_not_the_editor_separator() {
        assert_eq!(context("Code", "my-project - notes"), "my-project - notes");
    }

    #[test]
    fn a_dangling_editor_separator_falls_back_to_the_whole_title() {
        assert_eq!(context("Code", "file.rs — "), "file.rs —");
    }

    #[test]
    fn other_editors_follow_the_same_rule_whatever_the_case() {
        assert_eq!(context("zed", "lib.rs — vitarchy"), "vitarchy");
        assert_eq!(context("Visual Studio Code", "a.ts — web"), "web");
    }

    #[test]
    fn a_browser_title_with_a_known_site_suffix_gives_the_site() {
        assert_eq!(
            context("Brave Browser", "Lofi beats 2 hours - YouTube"),
            "YouTube"
        );
        assert_eq!(context("Brave Browser", "New chat - Claude"), "Claude");
        assert_eq!(
            context("Google Chrome", "Two Sum | takeUforward"),
            "takeUforward"
        );
        assert_eq!(
            context("Safari", "Pull requests · owner/repo · GitHub"),
            "GitHub"
        );
        assert_eq!(context("Arc", "Rust vs Go - reddit - Reddit"), "Reddit");
    }

    #[test]
    fn a_site_matches_without_regard_to_case_and_on_its_bare_name() {
        assert_eq!(context("Firefox", "lofi - youtube"), "YouTube");
        assert_eq!(context("Firefox", "YouTube"), "YouTube");
        assert_eq!(context("Firefox", "Claude"), "Claude");
    }

    #[test]
    fn a_tab_playing_audio_still_names_its_site() {
        assert_eq!(
            context("Brave Browser", "Lofi beats - YouTube 🔊"),
            "YouTube"
        );
        assert_eq!(
            context("Brave Browser", "Lofi beats - YouTube 🔇"),
            "YouTube"
        );
    }

    #[test]
    fn a_github_repository_path_is_github() {
        assert_eq!(
            context("Brave Browser", "khatrihemang07/meologue_01"),
            "GitHub"
        );
        assert_eq!(
            context("Brave Browser", "owner/repo: a short description"),
            "GitHub"
        );
    }

    #[test]
    fn prose_with_a_slash_is_not_a_repository_path() {
        assert_eq!(
            context("Brave Browser", "and/or usage in writing"),
            OTHER_PAGES
        );
        assert_eq!(context("Brave Browser", "a/b/c"), OTHER_PAGES);
    }

    #[test]
    fn a_browser_page_of_no_known_site_is_other_pages() {
        assert_eq!(
            context("Brave Browser", "Transactions · fin_02"),
            OTHER_PAGES
        );
        assert_eq!(context("Brave Browser", "How to bake bread"), OTHER_PAGES);
    }

    #[test]
    fn an_empty_title_gives_the_app_for_every_kind_of_app() {
        assert_eq!(context("Code", ""), "Code");
        assert_eq!(context("Brave Browser", "  "), "Brave Browser");
        assert_eq!(context("Spotify", ""), "Spotify");
    }

    #[test]
    fn every_other_app_is_its_own_context() {
        assert_eq!(context("Spotify", "Some Song - Some Artist"), "Spotify");
        assert_eq!(context("Finder", "Downloads"), "Finder");
    }

    #[test]
    fn invisible_marks_around_an_app_name_are_trimmed() {
        assert_eq!(clean_app("\u{200E}WhatsApp"), "WhatsApp");
        assert_eq!(clean_app(" \u{FEFF}Spotify\u{200B} "), "Spotify");
        assert_eq!(context("\u{200E}Brave Browser", "x - YouTube"), "YouTube");
    }

    #[test]
    fn the_repository_path_check_accepts_only_owner_slash_repo() {
        assert!(is_repo_path("khatrihemang07/meologue_01"));
        assert!(is_repo_path("a-b/c.d"));
        assert!(!is_repo_path("no-slash"));
        assert!(!is_repo_path("/repo"));
        assert!(!is_repo_path("owner/"));
        assert!(!is_repo_path("two words/repo"));
    }
}
