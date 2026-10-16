import type { components } from "../schema";

/** What `GET /api/time/status` returns. */
export type TimeStatus = components["schemas"]["TimeStatus"];
/** One Sync or Import attempt. */
export type TimeRun = components["schemas"]["RunInfo"];
/** What a Sync did. */
export type SyncCounts = components["schemas"]["SyncCounts"];

/** What an Import did. */
export type ImportCounts = components["schemas"]["ImportCounts"];

/** Imported Activity, the Snapshot and the latest Import. */
export type ActivityStatus = components["schemas"]["ActivityStatus"];

/** Screen time by app, Context and title. */
export type ActivityTotals = components["schemas"]["Totals"];
/** One app's time. */
export type ActivityAppTotal = components["schemas"]["AppTotal"];
/** One Context's time within an app. */
export type ActivityContextTotal = components["schemas"]["ContextTotal"];
/** Seconds by weekday (Monday first) and hour. */
export type ActivityHeatmap = components["schemas"]["Heatmap"];
/** First and last Activity of one Day. */
export type ActivityDayBound = components["schemas"]["DayBound"];
/** One Day's lane of Activity spans. */
export type ActivityTimeline = components["schemas"]["DayTimeline"];
/** One stretch of Activity in one Context. */
export type ActivitySpan = components["schemas"]["Span"];
/** Weekly screen time for the busiest apps. */
export type ActivityTrend = components["schemas"]["Trend"];
