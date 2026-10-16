//! The data behind Context derivation: which apps are editors or browsers, and
//! which sites a browser title can name. Add a row here to teach Vitarchy a new
//! app or site; `context.rs` holds the logic and never changes for that.

use super::context::{browser_site, editor_workspace};

/// How to read a title for the apps in one row.
pub struct AppRule {
    /// App names (compared ignoring case).
    pub apps: &'static [&'static str],
    /// The Context a non-empty title names.
    pub extract: fn(&str) -> String,
}

/// Editors name their workspace; browsers name their site.
pub const APP_RULES: &[AppRule] = &[
    AppRule {
        apps: &[
            "Code",
            "Visual Studio Code",
            "Code - Insiders",
            "VSCodium",
            "Cursor",
            "Windsurf",
            "Zed",
            "Xcode",
            "Sublime Text",
            "Nova",
            "IntelliJ IDEA",
            "PyCharm",
            "WebStorm",
            "GoLand",
            "RustRover",
            "CLion",
            "Android Studio",
            "Fleet",
        ],
        extract: editor_workspace,
    },
    AppRule {
        apps: &[
            "Brave Browser",
            "Safari",
            "Google Chrome",
            "Chromium",
            "Arc",
            "Firefox",
            "Microsoft Edge",
            "Opera",
            "Vivaldi",
            "Orion",
            "ego",
            "ego lite",
        ],
        extract: browser_site,
    },
];

/// Separates a file from its workspace in an editor title (an em dash with spaces).
pub const EDITOR_SEPARATOR: &str = " — ";

/// Speaker marks a browser appends to the title of a tab that is playing or muted.
pub const AUDIO_MARKS: &[char] = &['🔊', '🔇', '\u{FE0F}', ' '];

/// The Context of a browser page that matches no known site.
pub const OTHER_PAGES: &str = "Other pages";

/// One way a title can name a site.
pub enum Matcher {
    /// The title ends with this text (compared ignoring case).
    Suffix(&'static str),
    /// The title is exactly this text (compared ignoring case).
    Exact(&'static str),
    /// The title starts with a GitHub `owner/repo` path.
    RepoPath,
}

/// A site and the ways its pages are recognised.
pub struct Site {
    /// What the Context is called.
    pub name: &'static str,
    /// Any one matching names the site.
    pub matchers: &'static [Matcher],
}

use Matcher::{Exact, RepoPath, Suffix};

/// Known sites, tried in order.
pub const SITES: &[Site] = &[
    Site {
        name: "YouTube",
        matchers: &[Suffix(" - YouTube"), Exact("YouTube")],
    },
    Site {
        name: "Claude",
        matchers: &[Suffix(" - Claude"), Exact("Claude")],
    },
    Site {
        name: "ChatGPT",
        matchers: &[Suffix(" - ChatGPT"), Suffix(" | ChatGPT"), Exact("ChatGPT")],
    },
    Site {
        name: "GitHub",
        matchers: &[
            Suffix(" · GitHub"),
            Suffix(" - GitHub"),
            Suffix(" | GitHub"),
            Exact("GitHub"),
            RepoPath,
        ],
    },
    Site {
        name: "takeUforward",
        matchers: &[
            Suffix(" | takeUforward"),
            Suffix(" - takeUforward"),
            Suffix(" | TUF+"),
            Exact("takeUforward"),
            Exact("TUF+"),
        ],
    },
    Site {
        name: "meologue",
        matchers: &[Exact("meologue")],
    },
    Site {
        name: "Gemini",
        matchers: &[Suffix(" - Google Gemini"), Exact("Google Gemini")],
    },
    Site {
        name: "Google Drive",
        matchers: &[Suffix(" - Google Drive"), Exact("Google Drive")],
    },
    Site {
        name: "LeetCode",
        matchers: &[Suffix(" - LeetCode"), Exact("LeetCode")],
    },
    Site {
        name: "Google",
        matchers: &[Suffix(" - Google Search"), Exact("Google")],
    },
    Site {
        name: "Gmail",
        matchers: &[Suffix(" - Gmail"), Exact("Gmail")],
    },
    Site {
        name: "Google Docs",
        matchers: &[
            Suffix(" - Google Docs"),
            Suffix(" - Google Sheets"),
            Suffix(" - Google Slides"),
        ],
    },
    Site {
        name: "Google Meet",
        matchers: &[Suffix(" - Google Meet"), Exact("Google Meet")],
    },
    Site {
        name: "Stack Overflow",
        matchers: &[Suffix(" - Stack Overflow"), Exact("Stack Overflow")],
    },
    Site {
        name: "Wikipedia",
        matchers: &[Suffix(" - Wikipedia"), Exact("Wikipedia")],
    },
    Site {
        name: "Reddit",
        matchers: &[Suffix(" - Reddit"), Exact("Reddit")],
    },
    Site {
        name: "X",
        matchers: &[Suffix(" / X"), Exact("X")],
    },
    Site {
        name: "LinkedIn",
        matchers: &[Suffix(" | LinkedIn"), Exact("LinkedIn")],
    },
    Site {
        name: "Notion",
        matchers: &[Suffix(" | Notion"), Exact("Notion")],
    },
    Site {
        name: "Netflix",
        matchers: &[Suffix(" - Netflix"), Exact("Netflix")],
    },
    Site {
        name: "Instagram",
        matchers: &[
            Suffix(" • Instagram photos and videos"),
            Suffix(" | Instagram"),
            Exact("Instagram"),
        ],
    },
    Site {
        name: "WhatsApp Web",
        matchers: &[Exact("WhatsApp"), Suffix(" - WhatsApp")],
    },
    Site {
        name: "Hacker News",
        matchers: &[Suffix(" | Hacker News"), Exact("Hacker News")],
    },
    Site {
        name: "Medium",
        matchers: &[Suffix(" | Medium"), Suffix(" - Medium")],
    },
];
