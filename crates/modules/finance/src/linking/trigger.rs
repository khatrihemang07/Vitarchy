//! Why a detection run started, for the log.

/// What started a detection run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    /// A Statement was confirmed.
    Confirm,
    /// One Statement was read again.
    ReadAgain,
    /// Every Statement was read again.
    ReadAllAgain,
    /// Duplicate Accounts were merged.
    Merge,
    /// An Account was shown again.
    Unhide,
    /// The one-time run at server start.
    Backfill,
}

impl Trigger {
    /// The label that goes in the log.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Confirm => "confirm",
            Self::ReadAgain => "read_again",
            Self::ReadAllAgain => "read_all_again",
            Self::Merge => "merge",
            Self::Unhide => "unhide",
            Self::Backfill => "backfill",
        }
    }
}

/// What started a detection run, and the request it happened in (if any), so the run's log
/// line can be found from the request that caused it.
#[derive(Debug, Clone, Copy)]
pub struct Cause<'a> {
    /// What happened.
    pub trigger: Trigger,
    /// The request's id; `None` outside a request (server start, the repair command).
    pub request_id: Option<&'a str>,
}

impl<'a> Cause<'a> {
    /// A run started by `trigger` inside the request `request_id`.
    #[must_use]
    pub const fn in_request(trigger: Trigger, request_id: Option<&'a str>) -> Self {
        Self {
            trigger,
            request_id,
        }
    }

    /// A run started outside any request.
    #[must_use]
    pub const fn outside_request(trigger: Trigger) -> Self {
        Self {
            trigger,
            request_id: None,
        }
    }
}
