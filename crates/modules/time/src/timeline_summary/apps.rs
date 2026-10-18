//! Apps ranked by Active time, with their window titles and sites.

use std::collections::BTreeMap;

use super::{
    clip::Piece,
    domain::domain,
    model::{AppUsage, SiteUsage, TitleUsage},
};

/// Most window titles, and most sites, kept per app.
pub const TOP_PER_APP: usize = 10;

/// Apps ranked by Active seconds, most first; ties break by name. Away and unknown time is
/// left out. Each app's titles and sites are ranked the same way (a site is a url's domain).
#[must_use]
#[allow(clippy::cast_precision_loss)] // seconds are far below 2^52
pub fn rank(pieces: &[Piece]) -> Vec<AppUsage> {
    let mut apps: BTreeMap<&str, Tally> = BTreeMap::new();
    for piece in pieces.iter().filter(|p| p.is_active()) {
        let tally = apps.entry(&piece.app).or_default();
        tally.secs += piece.secs();
        if !piece.title.is_empty() {
            *tally.titles.entry(piece.title.clone()).or_default() += piece.secs();
        }
        if let Some(site) = piece.url.as_deref().and_then(domain) {
            *tally.sites.entry(site).or_default() += piece.secs();
        }
    }
    let total: i64 = apps.values().map(|t| t.secs).sum();
    let mut ranked: Vec<AppUsage> = apps
        .into_iter()
        .map(|(app, tally)| AppUsage {
            app: app.to_owned(),
            active_secs: tally.secs,
            share: tally.secs as f64 / total as f64,
            titles: top(tally.titles)
                .into_iter()
                .map(|(title, active_secs)| TitleUsage { title, active_secs })
                .collect(),
            sites: top(tally.sites)
                .into_iter()
                .map(|(domain, active_secs)| SiteUsage {
                    domain,
                    active_secs,
                })
                .collect(),
        })
        .collect();
    // The map already orders by name, and the sort is stable, so ties stay by name.
    ranked.sort_by_key(|a| std::cmp::Reverse(a.active_secs));
    ranked
}

/// Active seconds of one app, in total and by title and site.
#[derive(Default)]
struct Tally {
    secs: i64,
    titles: BTreeMap<String, i64>,
    sites: BTreeMap<String, i64>,
}

/// The [`TOP_PER_APP`] entries with the most seconds; ties by name (the map's order).
fn top(counts: BTreeMap<String, i64>) -> Vec<(String, i64)> {
    let mut all: Vec<_> = counts.into_iter().collect();
    all.sort_by_key(|(_, secs)| std::cmp::Reverse(*secs));
    all.truncate(TOP_PER_APP);
    all
}
