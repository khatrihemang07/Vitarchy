//! `GET /api/time/report` and `GET /api/time/facets`: the Reports page's data.

use axum::{
    Json,
    extract::{RawQuery, State},
};
use chrono::Utc;
use utoipa::IntoParams;
use vitarchy_core::{ApiError, AppState, Problem};

use super::previous_period;
use crate::{
    repo::{catalog, entries},
    report::{
        Context, Entries, Reference, build_report, facets::Facets, grouping::Grouping,
        model::Report, period::PeriodKind, query::ReportQuery,
    },
};

/// The query of `GET /api/time/report`. Documentation only: axum's `Query` cannot read a
/// repeated name (`project=1&project=2`), so the handler reads the raw query itself.
#[derive(IntoParams)]
#[into_params(parameter_in = Query)]
#[allow(dead_code)]
pub struct ReportParams {
    /// How the Period was chosen. `day` needs `from == to`, `week` 7 Days, `month` a whole
    /// calendar month.
    kind: PeriodKind,
    /// First Day, `YYYY-MM-DD`, in `tz`.
    from: String,
    /// Last Day (inclusive), `YYYY-MM-DD`; at most 366 Days after `from`.
    to: String,
    /// IANA time zone of the person, e.g. `Europe/Berlin`.
    tz: String,
    /// Only entries on these project ids (repeat the name). OR within, AND with other filters.
    project: Option<Vec<i64>>,
    /// Only entries with one of these tag names (repeat the name).
    tag: Option<Vec<String>>,
    /// Only entries whose project belongs to one of these client ids (repeat the name).
    client: Option<Vec<i64>>,
    /// Only descriptions containing this text, ignoring case.
    q: Option<String>,
    /// First-level grouping; defaults to `description`.
    group1: Option<Grouping>,
    /// Optional second-level grouping; must differ from `group1`.
    group2: Option<Grouping>,
}

/// A Report for one Period: overview, daily totals and groups, compared with the Previous Period
/// and with the Usual (the Usual's range before the Period).
#[utoipa::path(
    get,
    path = "/report",
    tag = "time",
    params(ReportParams),
    responses(
        (status = 200, description = "The Report", body = Report),
        (status = 400, description = "Invalid Period, time zone, filter or grouping", body = Problem),
        (status = 500, description = "Internal error", body = Problem),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn get_report(
    State(state): State<AppState>,
    RawQuery(raw): RawQuery,
) -> Result<Json<Report>, ApiError> {
    let pairs: Vec<(String, String)> =
        url::form_urlencoded::parse(raw.unwrap_or_default().as_bytes())
            .into_owned()
            .collect();
    let query = ReportQuery::parse(&pairs).map_err(|e| ApiError::BadRequest(e.to_string()))?;
    let (from, to) = query.period.utc_range();
    let current = entries::in_range(&state.pool, from, to).await?;
    let (previous_period, previous_entries) = previous_period::resolve(&state.pool, &query).await?;
    let (usual_from, usual_to) = query.period.usual_range().utc_range();
    let usual = entries::in_range(&state.pool, usual_from, usual_to).await?;
    // Ignores the filters: it is about how long the person has tracked, not what they look at.
    let first_entry = entries::first_start(&state.pool).await?;
    let entries = Entries {
        current: &current,
        previous: Reference {
            period: previous_period,
            entries: &previous_entries,
        },
        usual: &usual,
    };
    let context = Context {
        now: Utc::now(),
        first_entry,
    };
    let report = build_report(&query, &entries, &context);
    tracing::info!(
        kind = ?query.period.kind,
        from = %query.period.from,
        to = %query.period.to,
        filters = ?query.filter.counts(),
        groupings = query.groupings().len(),
        entries_read = current.len(),
        previous_entries_read = previous_entries.len(),
        previous_skipped = report.period.previous_skipped,
        usual_entries_read = usual.len(),
        elapsed_days = report.period.elapsed_days,
        usual_available_days = report.period.usual_available_days,
        groups = report.groups.len(),
        "report built"
    );
    Ok(Json(report))
}

/// Projects, tags and clients that have Time Entries, newest use first, for the filter pickers.
#[utoipa::path(
    get,
    path = "/facets",
    tag = "time",
    responses(
        (status = 200, description = "What the filters can offer", body = Facets),
        (status = 500, description = "Internal error", body = Problem),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn get_facets(State(state): State<AppState>) -> Result<Json<Facets>, ApiError> {
    let facets = catalog::used_facets(&state.pool).await?;
    tracing::info!(
        projects = facets.projects.len(),
        tags = facets.tags.len(),
        clients = facets.clients.len(),
        "facets read"
    );
    Ok(Json(facets))
}
