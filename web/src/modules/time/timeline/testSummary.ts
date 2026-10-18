import type { TimelineSummary, UsualFigure } from "@/api/time";

const figure = (avg: number, scaled: number | null = avg * 7): UsualFigure => ({ per_day: { avg_s: avg, min_s: 0, max_s: avg * 2 }, scaled_total: scaled });

/** A Timeline summary for tests; override any part. */
export function summary(over: Partial<TimelineSummary> = {}, kind: "day" | "week" = "week"): TimelineSummary {
  const isDay = kind === "day";
  return {
    period: { kind, from: "2026-10-12", to: isDay ? "2026-10-12" : "2026-10-18", usual_from: "2026-09-14", usual_to: "2026-10-11", elapsed_days: isDay ? 1 : 7, usual_available_days: 28 },
    overview: { active_secs: 18_000, away_secs: 3600, unknown_secs: 1800, switches: 40, switches_per_active_hour: 8, longest_focus_secs: 2700, focus_share: 0.4, first_active: "2026-10-12T07:00:00Z", last_active: "2026-10-12T17:00:00Z" },
    usual: { days: 20, active: figure(3600, 25_200), away: figure(600, 4200), unknown: figure(300, 2100), switches: figure(5, 35), longest_focus: figure(1800, null) },
    days: [{ date: "2026-10-12", active_secs: 18_000, away_secs: 3600, unknown_secs: 1800 }],
    hours: Array.from({ length: 24 }, (_, h) => (h >= 8 && h < 17 ? 1800 : 0)),
    apps: [
      { app: "Code", active_secs: 10_000, share: 0.55, titles: [{ title: "main.rs", active_secs: 6000 }], sites: [] },
      { app: "Safari", active_secs: 8000, share: 0.45, titles: [{ title: "Docs", active_secs: 5000 }], sites: [{ domain: "docs.rs", active_secs: 4000 }] },
    ],
    segments: isDay
      ? [
          { start_at: "2026-10-12T08:00:00Z", end_at: "2026-10-12T09:00:00Z", app: "Code", title: "main.rs", url: null, away: "no", sources: ["toggl"] },
          { start_at: "2026-10-12T09:00:00Z", end_at: "2026-10-12T09:30:00Z", app: "Safari", title: "Docs", url: "https://docs.rs/x", away: "no", sources: ["clockify"] },
          { start_at: "2026-10-12T09:30:00Z", end_at: "2026-10-12T10:00:00Z", app: "", title: "", url: null, away: "yes", sources: ["clockify"] },
        ]
      : null,
    entries: isDay ? [{ start_at: "2026-10-12T08:00:00Z", stop_at: "2026-10-12T10:00:00Z", description: "Build", project: "Office" }] : null,
    ...over,
  };
}
