//! The Usual: how the Timeline looked over the Usual's range.

use serde::Serialize;
use utoipa::ToSchema;

use crate::report::stats::Spread;

/// One figure of the Usual.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
pub struct UsualFigure {
    /// Average, shortest and longest per Day, over the Days that have Timeline.
    pub per_day: Spread,
    /// The figure's total over those Days scaled to the Days that have passed in the Period;
    /// absent for figures that do not add up across Days.
    pub scaled_total: Option<i64>,
}

/// How the Timeline usually looked over the Usual's range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
pub struct UsualTimeline {
    /// Days in the Usual's range that have Timeline.
    pub days: i64,
    /// Active seconds.
    pub active: UsualFigure,
    /// Away seconds.
    pub away: UsualFigure,
    /// Seconds whose Away is unknown.
    pub unknown: UsualFigure,
    /// Switches.
    pub switches: UsualFigure,
    /// The longest Focus stretch of each Day, in seconds.
    pub longest_focus: UsualFigure,
}
