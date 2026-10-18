//! `GET /api/time/timeline`: the Timeline page's data for one Period.

use std::time::Instant;

use axum::{
    Json,
    extract::{RawQuery, State},
};
use chrono::Utc;
use utoipa::IntoParams;
use vitarchy_core::{ApiError, AppState, Problem};

use crate::{
    error::TimeError,
    repo::{entries, timeline_read},
    report::{
        period::PeriodKind,
        period_query::{Pairs, parse_period},
    },
    timeline::{TimeRange, ensure_rules_current_or_warn},
    timeline_summary::{model::TimelineSummary, summarise},
};

/// The query of `GET /api/time/timeline`. Documentation only: the handler reads the raw query
/// with the same parser as Reports.
#[derive(IntoParams)]
#[into_params(parameter_in = Query)]
#[allow(dead_code)]
pub struct TimelineParams {
    /// How the Period was chosen. `day` needs `from == to`, `week` 7 Days, `month` a whole
    /// calendar month.
    kind: PeriodKind,
    /// First Day, `YYYY-MM-DD`, in `tz`.
    from: String,
    /// Last Day (inclusive), `YYYY-MM-DD`; at most 366 Days after `from`.
    to: String,
    /// IANA time zone of the person, e.g. `Europe/Berlin`.
    tz: String,
}

/// The Timeline of one Period, analysed and compared with the Usual. A Day also lists its
/// Segments and Time Entries.
#[utoipa::path(
    get,
    path = "/timeline",
    tag = "time",
    params(TimelineParams),
    responses(
        (status = 200, description = "The Timeline summary", body = TimelineSummary),
        (status = 400, description = "Invalid Period or time zone", body = Problem),
        (status = 500, description = "Internal error", body = Problem),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn get_timeline(
    State(state): State<AppState>,
    RawQuery(raw): RawQuery,
) -> Result<Json<TimelineSummary>, TimeError> {
    let started = Instant::now();
    ensure_rules_current_or_warn(&state.pool).await;
    let pairs: Vec<(String, String)> =
        url::form_urlencoded::parse(raw.unwrap_or_default().as_bytes())
            .into_owned()
            .collect();
    let period = parse_period(&Pairs(&pairs)).map_err(|e| ApiError::BadRequest(e.to_string()))?;
    let (from, to) = period.utc_range();
    let (usual_from, usual_to) = period.usual_range().utc_range();
    let range = TimeRange { from, to }.union(TimeRange {
        from: usual_from,
        to: usual_to,
    });
    let segments = timeline_read::in_range(&state.pool, range).await?;
    let first_start = timeline_read::first_start(&state.pool).await?;
    let lane = if period.kind == PeriodKind::Day {
        entries::in_range(&state.pool, from, to).await?
    } else {
        Vec::new()
    };
    let summary = summarise(&period, &segments, &lane, Utc::now(), first_start);
    tracing::info!(
        kind = ?period.kind,
        days = summary.days.len(),
        segments = segments.len(),
        duration_ms = started.elapsed().as_millis(),
        "timeline summary built"
    );
    Ok(Json(summary))
}
