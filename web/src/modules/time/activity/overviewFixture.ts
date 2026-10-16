/** Test helper: a full overview response with every part empty, overridden part by part. */
export const OVERVIEW = "GET /api/time/analysis/activity/overview";

/** An overview body for the stubbed API. */
export function overviewBody(parts: Record<string, unknown> = {}) {
  return {
    totals: { total_seconds: 0, apps: [] },
    heatmap: { seconds: Array.from({ length: 7 }, () => Array.from({ length: 24 }, () => 0)) },
    bounds: [],
    trend: { weeks: [], series: [], other: [] },
    ...parts,
  };
}
