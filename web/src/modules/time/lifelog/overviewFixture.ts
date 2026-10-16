/** Test helper: the Life log overview route and a full response, overridden part by part. */
export const LIFELOG_OVERVIEW = "GET /api/time/analysis/lifelog/overview";
export const LIFELOG_TIMELINE = "GET /api/time/analysis/lifelog/timeline";

/** An overview body for the stubbed API with nothing logged. */
export function lifeLogBody(parts: Record<string, unknown> = {}) {
  return {
    split: [{ date: "2026-05-04", elapsed_seconds: 86400, unlogged_seconds: 86400, by_project: [] }],
    totals: { logged_seconds: 0, by_project: [], by_description: [] },
    description_stats: [],
    ...parts,
  };
}

const description = (name: string, seconds: number, entries: number, days: number) => ({ description: name, seconds, entries, days });

/** A small logged life: an office Day and a night's sleep. */
export const LOGGED = lifeLogBody({
  split: [
    { date: "2026-05-04", elapsed_seconds: 86400, unlogged_seconds: 50400, by_project: [{ project: "office", seconds: 28800 }, { project: "sleep", seconds: 7200 }] },
    { date: "2026-05-05", elapsed_seconds: 86400, unlogged_seconds: 79200, by_project: [{ project: "sleep", seconds: 25200 }] },
  ],
  totals: {
    logged_seconds: 61200,
    by_project: [
      { name: "office", color: "#2a78d6", seconds: 28800, entries: 1, days: 1, descriptions: [description("inside_office", 28800, 1, 1)] },
      { name: "sleep", color: null, seconds: 32400, entries: 1, days: 2, descriptions: [description("sleep", 32400, 1, 2)] },
    ],
    by_description: [description("sleep", 32400, 1, 2), description("inside_office", 28800, 1, 1)],
  },
  description_stats: [
    {
      description: "sleep",
      days: 6,
      entries: 6,
      start: { median_minute: 0, p25_minute: 1420, p75_minute: 25 },
      end: { median_minute: 450, p25_minute: 450, p75_minute: 450 },
      duration: { median_seconds: 27000, p25_seconds: 25200, p75_seconds: 28800 },
    },
  ],
});
