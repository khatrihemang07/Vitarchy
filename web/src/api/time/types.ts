import type { components, paths } from "../schema";

/** What `GET /api/time/status` returns. */
export type TimeStatus = components["schemas"]["TimeStatus"];
/** One Sync or Import attempt. */
export type TimeRun = components["schemas"]["RunInfo"];
/** What a Sync did. */
export type SyncCounts = components["schemas"]["SyncCounts"];

/** What an Import did. */
export type ImportCounts = components["schemas"]["ImportCounts"];

/** One Source's stored Activity, Snapshot and latest Import. */
export type ActivityStatus = components["schemas"]["ActivityStatus"];
/** An Activity Source: a tracking app whose Activity is Imported. */
export type Source = components["schemas"]["Source"];
/** [`ActivityStatus`] naming its Source. */
export type SourceActivityStatus = components["schemas"]["SourceActivityStatus"];
/** How a Period was chosen: one Day, a week, a month or a custom stretch of Days. */
export type PeriodKind = components["schemas"]["PeriodKind"];
/** A way to group Time Entries: by description, project or tag. */
export type Grouping = components["schemas"]["Grouping"];
/** What `GET /api/time/report` returns. */
export type Report = components["schemas"]["Report"];
/** One row of a Report: a group with its stats, daily totals and children or entries. */
export type ReportGroup = components["schemas"]["Group"];
/** A Time Entry listed under a bottom-level group. */
export type ReportEntry = components["schemas"]["ReportEntry"];
/** Days, entries, total and the per-Day and per-entry spreads of a group. */
export type ReportStats = components["schemas"]["Stats"];
/** The change of a group's stats from the Previous Period. */
export type ReportStatsChange = components["schemas"]["StatsChange"];
/** Seconds tracked on one Day. */
export type DailyPoint = components["schemas"]["DailyPoint"];
/** What `GET /api/time/facets` returns: the choices for the filters. */
export type Facets = components["schemas"]["Facets"];
/** A project that has Time Entries. */
export type ProjectFacet = components["schemas"]["ProjectFacet"];
/** Query of `GET /api/time/report`. */
export type ReportQuery = NonNullable<paths["/api/time/report"]["get"]["parameters"]["query"]>;

/** The stored Timeline's figures in the Time status. */
export type TimelineStatus = components["schemas"]["TimelineStatus"];
/** What a Timeline rebuild did. */
export type RebuildResult = components["schemas"]["RebuildResult"];
