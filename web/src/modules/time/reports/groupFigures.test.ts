import type { ReportGroup } from "@/api/time";
import { figuresFor } from "./groupFigures";

const spread = (a: number, b: number, c: number) => ({ avg_s: a, min_s: b, max_s: c });
const change = {
  days: 1, entries: -1, total_s: 60,
  per_day: { avg_s: 10, min_s: 0, max_s: -20 }, per_entry: { avg_s: 5, min_s: 1, max_s: 2 },
};
const usualChange = {
  days: 2, entries: 3, total_s: 900,
  per_day: { avg_s: -2520, min_s: 7, max_s: 8 }, per_entry: { avg_s: 70, min_s: 71, max_s: 72 },
};
const usual = { days: 9, per_day: spread(25800, 18000, 32000), per_entry: spread(900, 300, 1800) };

const group = (over: Partial<ReportGroup> = {}): ReportGroup => ({
  key: "k", label: "L", children: [], entries: [], daily: [],
  stats: { days: 3, entries: 5, total_s: 1000, per_day: spread(300, 100, 500), per_entry: spread(200, 50, 400) },
  vs_previous: { is_new: false, change },
  vs_usual: { is_new: false, change: usualChange, usual },
  ...over,
});

describe("figuresFor", () => {
  it("reads the per-Day spread in per_day mode", () => {
    expect(figuresFor(group(), "per_day", "previous")).toMatchObject({
      days: 3, entries: 5, total: { value: 1000, change: 60 },
      avg: { value: 300, change: 10 }, min: { value: 100, change: 0 }, max: { value: 500, change: -20 },
    });
  });

  it("reads the per-entry spread in per_entry mode", () => {
    expect(figuresFor(group(), "per_entry", "previous")).toMatchObject({
      avg: { value: 200, change: 5 }, min: { value: 50, change: 1 }, max: { value: 400, change: 2 },
    });
  });

  it("takes the changes from the Usual comparison when comparing with the last 4 weeks", () => {
    const figures = figuresFor(group(), "per_day", "usual");
    expect(figures.total.change).toBe(900);
    expect(figures.avg.change).toBe(-2520);
    expect(figuresFor(group(), "per_entry", "usual").avg.change).toBe(70);
  });

  it("compares a Day's total with the Usual's average per Day, not with its scaled total", () => {
    expect(figuresFor(group(), "per_day", "usual").dayTotal).toEqual({ value: 1000, change: -2520 });
  });

  it("compares a Day's total with yesterday's total against the Previous Period", () => {
    expect(figuresFor(group(), "per_day", "previous").dayTotal).toEqual({ value: 1000, change: 60 });
  });

  it("offers the Usual's per-Day spread, whichever stat mode is chosen", () => {
    expect(figuresFor(group(), "per_entry", "usual").usual).toEqual({ avg: 25800, min: 18000, max: 32000 });
  });

  it("has no Usual figures against the Previous Period or for a group missing from the last 4 weeks", () => {
    expect(figuresFor(group(), "per_day", "previous").usual).toBeNull();
    expect(figuresFor(group({ vs_usual: { is_new: true, change: null } }), "per_day", "usual").usual).toBeNull();
  });

  it("has no changes for a group that is new in the chosen comparison only", () => {
    const mixed = group({ vs_usual: { is_new: true, change: null } });
    const against = figuresFor(mixed, "per_day", "usual");
    expect(against.isNew).toBe(true);
    expect(against.total.change).toBeNull();
    expect(against.avg.change).toBeNull();
    expect(against.dayTotal.change).toBeNull();
    expect(figuresFor(mixed, "per_day", "previous").isNew).toBe(false);
  });
});
