import type { ActivityTotals, LifeLogDaySplit } from "@/api/time";
import { describeChange, topApps, topContexts, unloggedShare } from "./overviewModel";

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

describe("topContexts", () => {
  it("lists the longest Contexts across apps, leaving out those that only repeat their app", () => {
    expect(topContexts(totals, 3).map((c) => c.name)).toEqual(["vitarchy", "YouTube", "Other pages"]);
  });

  it("adds up a Context that appears under two apps", () => {
    const twice: ActivityTotals = {
      total_seconds: 0,
      apps: [
        { app: "Brave", seconds: 0, contexts: [context("Docs", 100)] },
        { app: "Firefox", seconds: 0, contexts: [context("Docs", 250)] },
      ],
    };
    expect(topContexts(twice, 5)).toEqual([{ name: "Docs", seconds: 350 }]);
  });
});

describe("describeChange", () => {
  it("says how much more or less time there was than before", () => {
    expect(describeChange(3 * 3600, 2 * 3600)).toEqual({ direction: "up", text: "1 h more than the previous Period (+50%)" });
    expect(describeChange(2700, 3600)).toEqual({ direction: "down", text: "15 min less than the previous Period (-25%)" });
  });

  it("says when nothing changed, and when there is nothing to compare with", () => {
    expect(describeChange(3600, 3600)).toEqual({ direction: "same", text: "Same as the previous Period" });
    expect(describeChange(3600, 0)).toBeNull();
  });
});

describe("unloggedShare", () => {
  const day = (elapsed: number, unlogged: number): LifeLogDaySplit => ({ date: "2026-05-04", elapsed_seconds: elapsed, unlogged_seconds: unlogged, by_project: [] });

  it("adds up the Unlogged time and the time that has happened", () => {
    expect(unloggedShare([day(86400, 50400), day(43200, 3600)])).toEqual({ unlogged: 54000, elapsed: 129600 });
  });
});
