//! Pure: whose Toggl data the mirror holds, and when a new connection replaces it.

/// A Toggl user in one workspace: what the mirror is a copy of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Account {
    /// Toggl user id.
    pub user_id: i64,
    /// The synced workspace.
    pub workspace_id: i64,
}

/// True when connecting `new` makes what is mirrored for `owner` foreign data that
/// must go. Reconnecting the same account (a new token, say) keeps it; with no
/// recorded owner there is nothing known to be foreign.
#[must_use]
pub fn mirror_is_foreign(owner: Option<Account>, new: Account) -> bool {
    owner.is_some_and(|owner| owner != new)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ME: Account = Account {
        user_id: 5,
        workspace_id: 7,
    };

    #[test]
    fn the_same_account_keeps_the_mirror() {
        assert!(!mirror_is_foreign(Some(ME), ME));
    }

    #[test]
    fn another_user_or_another_workspace_replaces_it() {
        assert!(mirror_is_foreign(Some(ME), Account { user_id: 6, ..ME }));
        assert!(mirror_is_foreign(
            Some(ME),
            Account {
                workspace_id: 8,
                ..ME
            }
        ));
    }

    #[test]
    fn with_no_recorded_owner_nothing_is_foreign() {
        assert!(!mirror_is_foreign(None, ME));
    }
}
