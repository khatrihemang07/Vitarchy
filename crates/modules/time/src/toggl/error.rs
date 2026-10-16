//! Why a Toggl call failed, classified from the HTTP status so callers never
//! deal with raw status codes.

/// A failed call to Toggl. Carries no token and no payload, so it is safe to log.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TogglError {
    /// 401/403: Toggl does not accept the API token.
    #[error("Toggl rejected the API token")]
    InvalidToken,
    /// 402/429: the request quota is used up; `resets_in_secs` says when it recovers.
    #[error("Toggl request limit reached")]
    RateLimited {
        /// Seconds until the quota resets, from `X-Toggl-Quota-Resets-In`.
        resets_in_secs: Option<u64>,
    },
    /// The network call itself failed (offline, DNS, timeout).
    #[error("could not reach Toggl")]
    Unreachable,
    /// Any other non-success status.
    #[error("Toggl answered with status {0}")]
    Unexpected(u16),
    /// A success status whose body was not the expected shape.
    #[error("Toggl sent a response that could not be read")]
    Malformed,
}

impl TogglError {
    /// Classifies a response: `None` for a 2xx, otherwise the error it means.
    /// `resets_in` is the raw `X-Toggl-Quota-Resets-In` header, when present.
    #[must_use]
    pub fn from_status(status: u16, resets_in: Option<&str>) -> Option<Self> {
        match status {
            200..=299 => None,
            401 | 403 => Some(Self::InvalidToken),
            402 | 429 => Some(Self::RateLimited {
                resets_in_secs: resets_in.and_then(|v| v.trim().parse().ok()),
            }),
            other => Some(Self::Unexpected(other)),
        }
    }

    /// Stable machine label, stored in `runs.error_code` and in problem bodies.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidToken => "invalid_token",
            Self::RateLimited { .. } => "rate_limited",
            Self::Unreachable => "unreachable",
            Self::Unexpected(_) => "unexpected",
            Self::Malformed => "malformed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_statuses_are_not_errors() {
        assert_eq!(TogglError::from_status(200, None), None);
        assert_eq!(TogglError::from_status(204, None), None);
    }

    #[test]
    fn unauthorised_and_forbidden_mean_an_invalid_token() {
        for status in [401, 403] {
            assert_eq!(
                TogglError::from_status(status, None),
                Some(TogglError::InvalidToken)
            );
        }
    }

    #[test]
    fn payment_required_and_too_many_requests_are_rate_limits_with_the_reset_time() {
        for status in [402, 429] {
            assert_eq!(
                TogglError::from_status(status, Some(" 1800 ")),
                Some(TogglError::RateLimited {
                    resets_in_secs: Some(1800)
                })
            );
        }
    }

    #[test]
    fn a_rate_limit_without_a_readable_header_has_no_reset_time() {
        let expected = Some(TogglError::RateLimited {
            resets_in_secs: None,
        });
        assert_eq!(TogglError::from_status(429, None), expected);
        assert_eq!(TogglError::from_status(429, Some("soon")), expected);
    }

    #[test]
    fn other_statuses_are_unexpected() {
        assert_eq!(
            TogglError::from_status(500, None),
            Some(TogglError::Unexpected(500))
        );
    }

    #[test]
    fn every_error_has_its_own_code() {
        let codes = [
            TogglError::InvalidToken.code(),
            TogglError::RateLimited {
                resets_in_secs: None,
            }
            .code(),
            TogglError::Unreachable.code(),
            TogglError::Unexpected(500).code(),
            TogglError::Malformed.code(),
        ];
        let unique: std::collections::BTreeSet<_> = codes.iter().collect();
        assert_eq!(unique.len(), codes.len());
    }
}
