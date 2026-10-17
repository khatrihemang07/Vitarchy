import type { ActivityTotals, DayTotal, LifeLogDaySplit } from "@/api/time";
import { describeChange, topApps, unloggedShare } from "./overviewModel";

const context = (name: string, seconds: number) => ({ name, seconds, titles: [] });
const totals: ActivityTotals = {
  total_seconds: 20000,
  apps: [
    { app: "Brave", seconds: 9000, contexts: [context("YouTube", 5000), context("Other pages", 4000)] },
    { app: "Code", seconds: 8000, contexts: [context("vitarchy", 6000), context("notes", 2000)] },
    { app: "Slack", seconds: 3000, contexts: [context("Slack", 3000)] },
  ],
};

describe("topApps", () => {
  it("lists the longest apps first, up to the limit", () => {
    expect(topApps(totals, 2)).toEqual([
      { name: "Brave", seconds: 9000 },
      { name: "Code", seconds: 8000 },
    ]);
  });
});

describe("describeChange", () => {
  const days = (...hours: number[]): DayTotal[] => hours.map((h, i) => ({ date: `2026-05-${String(i + 1).padStart(2, "0")}`, seconds: h * 3600 }));

  it("compares the average per finished Day, so a Period part way through is not set against a whole one", () => {
    // Two Days of 1.5 h against seven of 2 h: less per day, though 3 h is nothing like 14 h.
    expect(describeChange(days(1.5, 1.5), days(2, 2, 2, 2, 2, 2, 2))).toEqual({ direction: "down", text: "30 min less per day than the previous Period (-25%)" });
    expect(describeChange(days(3, 3), days(2, 2, 2))).toEqual({ direction: "up", text: "1 h more per day than the previous Period (+50%)" });
  });

  it("says when nothing changed, and when either Period has no Day to compare", () => {
    expect(describeChange(days(1, 1), days(1, 1, 1))).toEqual({ direction: "same", text: "Same per day as the previous Period" });
    expect(describeChange(days(1), [])).toBeNull();
    expect(describeChange([], days(1))).toBeNull();
  });
});

describe("unloggedShare", () => {
  const day = (elapsed: number, unlogged: number): LifeLogDaySplit => ({ date: "2026-05-04", elapsed_seconds: elapsed, unlogged_seconds: unlogged, by_project: [] });

  it("adds up the Unlogged time and the time that has happened", () => {
    expect(unloggedShare([day(86400, 50400), day(43200, 3600)])).toEqual({ unlogged: 54000, elapsed: 129600 });
  });
});
