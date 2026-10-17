//! The group tree: entries grouped by one or two dimensions, each group with its numbers.

use std::collections::BTreeMap;

use chrono::NaiveDate;

use super::{
    grouping::{Grouping, most_common_label},
    items::Item,
    model::{Group, ReportEntry},
    series::daily_series,
    stats::{Part, compute},
};

/// Items of one group, with the labels they spelled it as.
#[derive(Default)]
struct Members<'a, 'b> {
    items: Vec<&'b Item<'a>>,
    labels: Vec<String>,
}

fn stats_of(items: &[&Item<'_>]) -> super::stats::Stats {
    let parts: Vec<Part> = items
        .iter()
        .flat_map(|i| {
            i.parts.iter().map(|p| Part {
                entry: i.entry.toggl_id,
                date: p.date,
                seconds: p.seconds,
            })
        })
        .collect();
    compute(&parts)
}

fn report_entry(item: &Item<'_>) -> ReportEntry {
    let e = item.entry;
    ReportEntry {
        toggl_id: e.toggl_id,
        start: e.start,
        stop: e.stop,
        duration_s: item.seconds(),
        description: e.description.clone(),
        project_id: e.project_id,
        project_name: e.project_name.clone(),
        tags: e.tags.clone(),
    }
}

fn group_of(
    key: String,
    members: &Members<'_, '_>,
    rest: &[Grouping],
    days: &[NaiveDate],
) -> Group {
    let leaf = rest.is_empty();
    let mut entries: Vec<ReportEntry> = if leaf {
        members.items.iter().map(|i| report_entry(i)).collect()
    } else {
        vec![]
    };
    entries.sort_by_key(|e| (e.start, e.toggl_id));
    Group {
        key,
        label: most_common_label(members.labels.iter().map(String::as_str)),
        stats: stats_of(&members.items),
        change: None,
        is_new: false,
        daily: daily_series(days, members.items.iter().flat_map(|i| i.parts.iter())),
        children: if leaf {
            vec![]
        } else {
            build(&members.items, rest, days)
        },
        entries,
    }
}

/// Groups `items` by the first of `groupings`, then each group by the rest. Largest first.
#[must_use]
pub fn build(items: &[&Item<'_>], groupings: &[Grouping], days: &[NaiveDate]) -> Vec<Group> {
    let Some((first, rest)) = groupings.split_first() else {
        return vec![];
    };
    let mut by_key: BTreeMap<String, Members<'_, '_>> = BTreeMap::new();
    for item in items {
        for keyed in first.keys(item.entry) {
            let members = by_key.entry(keyed.key).or_default();
            members.items.push(item);
            members.labels.push(keyed.label);
        }
    }
    let mut groups: Vec<Group> = by_key
        .into_iter()
        .map(|(key, members)| group_of(key, &members, rest, days))
        .collect();
    groups.sort_by(|a, b| {
        b.stats
            .total_s
            .cmp(&a.stats.total_s)
            .then_with(|| a.label.cmp(&b.label))
    });
    groups
}
