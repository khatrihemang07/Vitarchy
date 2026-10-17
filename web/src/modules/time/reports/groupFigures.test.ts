import type { ReportGroup } from "@/api/time";
import { figuresFor } from "./groupFigures";

const spread = (a: number, b: number, c: number) => ({ avg_s: a, min_s: b, max_s: c });
const group = (over: Partial<ReportGroup> = {}): ReportGroup => ({
  key: "k", label: "L", is_new: false, children: [], entries: [], daily: [],
  stats: { days: 3, entries: 5, total_s: 1000, per_day: spread(300, 100, 500), per_entry: spread(200, 50, 400) },
  change: {
    days: 1, entries: -1, total_s: 60,
    per_day: { avg_s: 10, min_s: 0, max_s: -20 }, per_entry: { avg_s: 5, min_s: 1, max_s: 2 },
  },
  ...over,
});

describe("figuresFor", () => {
  it("reads the per-Day spread in per_day mode", () => {
    expect(figuresFor(group(), "per_day")).toMatchObject({
      days: 3, entries: 5, total: { value: 1000, change: 60 },
      avg: { value: 300, change: 10 }, min: { value: 100, change: 0 }, max: { value: 500, change: -20 },
    });
  });

  it("reads the per-entry spread in per_entry mode", () => {
    expect(figuresFor(group(), "per_entry")).toMatchObject({ avg: { value: 200, change: 5 }, max: { value: 400, change: 2 } });
  });

  it("has no changes for a new group", () => {
    const figures = figuresFor(group({ is_new: true, change: null }), "per_day");
    expect(figures.isNew).toBe(true);
    expect(figures.total.change).toBeNull();
    expect(figures.avg.change).toBeNull();
  });
});
