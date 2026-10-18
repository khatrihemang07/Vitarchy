//! What the Timeline's merge needs to know about a Source: how well it records each trait,
//! and how its Activity becomes observations.

use super::model::Activity;
use crate::timeline::model::Observation;

/// Normalises one Activity into what the Timeline compares across Sources.
pub type Observe = fn(&Activity) -> Vec<Observation>;

/// How well a Source records each trait, as a rank where higher wins; `None` means it does
/// not record that trait at all. The Timeline's merge reads only these numbers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Strengths {
    /// Where one stretch ends and the next begins; also decides the app and title.
    pub boundaries: Option<u8>,
    /// The page address.
    pub url: Option<u8>,
    /// Whether the person was away.
    pub away: Option<u8>,
}
