//! Where Activity comes from. One descriptor per Source holds everything that differs
//! between them, so adding a Source is a new row here and a new reader, not new branches.

use std::path::Path;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{clockify, read_snapshot::SnapshotRows, toggl};

/// A tracking app whose Activity Vitarchy Imports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    /// Toggl Track for Mac.
    Toggl,
    /// Clockify Desktop.
    Clockify,
}

/// Reads a Snapshot file and parses its rows, on the calling (blocking) thread.
pub type Reader = fn(&Path) -> rusqlite::Result<SnapshotRows>;

/// Everything that differs between Sources.
pub struct Descriptor {
    /// The Source it describes.
    pub source: Source,
    /// Its id in URLs and the database; equals its serialized name.
    pub id: &'static str,
    /// Env var that replaces its Snapshot path (stage and tests).
    pub snapshot_env: &'static str,
    /// Where the helper writes its Snapshot, relative to the home folder. Restated in the
    /// helper crate and `scripts/lib/helper.sh`; `scripts/test-guards.sh` fails if they differ.
    pub default_relative: &'static str,
    /// Reads and parses its Snapshot.
    pub reader: Reader,
}

const TOGGL_RELATIVE: &str = "Library/Application Support/Vitarchy/activity/latest.sqlite";
const CLOCKIFY_RELATIVE: &str = "Library/Application Support/Vitarchy/clockify/latest.sqlite";

const TOGGL: Descriptor = Descriptor {
    source: Source::Toggl,
    id: "toggl",
    snapshot_env: "TIME_ACTIVITY_SNAPSHOT",
    default_relative: TOGGL_RELATIVE,
    reader: toggl::read::read_snapshot_rows,
};

const CLOCKIFY: Descriptor = Descriptor {
    source: Source::Clockify,
    id: "clockify",
    snapshot_env: "TIME_CLOCKIFY_SNAPSHOT",
    default_relative: CLOCKIFY_RELATIVE,
    reader: clockify::read::read_snapshot_rows,
};

/// Every Source, in the order the status lists them.
const ALL: [Source; 2] = [Source::Toggl, Source::Clockify];

impl Source {
    /// Every Source, in listing order.
    pub fn all() -> impl Iterator<Item = Self> {
        ALL.into_iter()
    }

    /// The Source whose id is `id`, if any.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::all().find(|source| source.id() == id)
    }

    /// This Source's descriptor. Exhaustive, so a new variant cannot compile without one.
    #[must_use]
    pub fn descriptor(self) -> &'static Descriptor {
        match self {
            Self::Toggl => &TOGGL,
            Self::Clockify => &CLOCKIFY,
        }
    }

    /// Its id in URLs and the database.
    #[must_use]
    pub fn id(self) -> &'static str {
        self.descriptor().id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_source_has_its_own_descriptor_in_listing_order() {
        let ids: Vec<_> = Source::all().map(Source::id).collect();
        assert_eq!(ids, ["toggl", "clockify"]);
        for source in Source::all() {
            assert_eq!(source.descriptor().source, source);
        }
    }

    #[test]
    fn the_id_is_the_serialized_name() {
        for source in Source::all() {
            assert_eq!(serde_json::to_value(source).unwrap(), source.id());
        }
    }

    #[test]
    fn an_id_finds_its_source_and_a_stranger_finds_none() {
        assert_eq!(Source::from_id("clockify"), Some(Source::Clockify));
        assert_eq!(Source::from_id("harvest"), None);
        assert_eq!(Source::from_id("Toggl"), None);
    }

    #[test]
    fn snapshot_env_vars_differ_between_sources() {
        let envs: Vec<_> = Source::all().map(|s| s.descriptor().snapshot_env).collect();
        assert_ne!(envs[0], envs[1]);
    }
}
