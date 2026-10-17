import type { Report, ReportGroup } from "@/api/time";

const spread = { avg_s: 3600, min_s: 1800, max_s: 7200 };

/** A bottom-level group with one entry, for tests. */
export function group(label: string, over: Partial<ReportGroup> = {}): ReportGroup {
  return {
    key: label.toLowerCase(), label, is_new: false, children: [], daily: [{ date: "2026-10-12", seconds: 3600 }],
    stats: { days: 1, entries: 1, total_s: 3600, per_day: spread, per_entry: spread },
    change: { days: 0, entries: 0, total_s: 900, per_day: { avg_s: 0, min_s: 0, max_s: 0 }, per_entry: { avg_s: 0, min_s: 0, max_s: 0 } },
    entries: [{ toggl_id: label.length, start: "2026-10-12T09:00:00Z", stop: "2026-10-12T10:00:00Z", duration_s: 3600, description: `${label} work`, project_name: "Office", tags: [] }],
    ...over,
  };
}

/** A week Report holding `groups`, for tests. */
export function report(groups: ReportGroup[], kind: Report["period"]["kind"] = "week"): Report {
  return {
    period: { kind, from: "2026-10-12", to: "2026-10-18", previous_from: "2026-10-05", previous_to: "2026-10-11" },
    overview: { total_s: 7200, entries: 2, days_tracked: 2, avg_per_day_s: 3600, change: { total_s: 600, entries: 0, days_tracked: 0, avg_per_day_s: 0 } },
    daily: [{ date: "2026-10-12", seconds: 7200 }],
    group1: "description", group2: null, groups,
  };
}
