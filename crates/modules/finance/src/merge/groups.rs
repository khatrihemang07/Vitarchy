//! Which Accounts are the same Account: same issuer, kind and visible digits.

use crate::accounts::{AccountIdentity, Match, find, remembered_digits};

/// Accounts that are one Account: `keep` is the oldest, `from` are merged into it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    /// The Account that stays.
    pub keep: i64,
    /// The duplicates, oldest first.
    pub from: Vec<i64>,
}

/// Groups `accounts` (ordered oldest first) that have duplicates. An Account with no visible
/// digits is never grouped: without a number it cannot be told from another.
#[must_use]
pub fn duplicate_groups(accounts: &[AccountIdentity]) -> Vec<Group> {
    let mut keepers: Vec<AccountIdentity> = Vec::new();
    let mut groups: Vec<Group> = Vec::new();
    for account in accounts {
        if remembered_digits(account.mask.as_deref()).is_none() {
            continue;
        }
        let found = find(
            Some(&account.issuer),
            Some(account.kind),
            account.mask.as_deref(),
            &keepers,
        );
        match found {
            Match::One(keep) => match groups.iter_mut().find(|g| g.keep == keep) {
                Some(group) => group.from.push(account.account_id),
                None => groups.push(Group {
                    keep,
                    from: vec![account.account_id],
                }),
            },
            // Several keepers fit: the person must decide; leave it alone.
            Match::Several(_) | Match::OneWithoutNumber(_) => {}
            Match::None => keepers.push(account.clone()),
        }
    }
    groups
}
