import { hasTimeline, overviewFigures, usualNote } from "./overviewFigures";
import { summary } from "./testSummary";

describe("overviewFigures", () => {
  it("lists Active, Away, Away unknown, Switches and Longest focus", () => {
    expect(overviewFigures(summary()).map((f) => f.label)).toEqual(["Active", "Away", "Away unknown", "Switches", "Longest focus"]);
  });

  it("compares totals with the Usual's scaled total", () => {
    const figures = overviewFigures(summary());
    expect(figures.find((f) => f.label === "Active")?.change).toBe(18_000 - 25_200);
    expect(figures.find((f) => f.label === "Switches")?.change).toBe(40 - 35);
  });

  it("compares Longest focus per Day: the average for a Day, the best Day for a longer Period", () => {
    expect(overviewFigures(summary({}, "day")).find((f) => f.label === "Longest focus")?.change).toBe(2700 - 1800);
    expect(overviewFigures(summary()).find((f) => f.label === "Longest focus")?.change).toBe(2700 - 3600);
  });

  it("says what Longest focus is compared with: your usual Day for a Day, your best Day for a longer Period", () => {
    const note = (data: Parameters<typeof overviewFigures>[0]) => overviewFigures(data).find((f) => f.label === "Longest focus")?.note;
    expect(note(summary({}, "day"))).toBe("vs your usual Day");
    expect(note(summary())).toBe("vs your best Day in the last 4 weeks");
    expect(note(summary({ usual: null }))).toBeUndefined();
  });

  it("has no change without a Usual", () => {
    expect(overviewFigures(summary({ usual: null })).every((f) => f.change === null)).toBe(true);
  });

  it("counts Switches, not time", () => {
    expect(overviewFigures(summary()).find((f) => f.label === "Switches")?.unit).toBe("count");
  });
});

describe("usualNote", () => {
  it("names the last 4 weeks and how many Days it could use", () => {
    expect(usualNote(summary())).toBe("Compared with the last 4 weeks, 14 Sep – 11 Oct 2026 (20 Days with Timeline)");
  });

  it("says why there is nothing to compare with", () => {
    expect(usualNote(summary({ usual: null }))).toMatch(/no history|nothing to compare/i);
  });
});

describe("hasTimeline", () => {
  it("is true when the Period has any covered time", () => {
    expect(hasTimeline(summary())).toBe(true);
  });
  it("is false for a Period with nothing in it", () => {
    const zero = { ...summary().overview, active_secs: 0, away_secs: 0, unknown_secs: 0 };
    expect(hasTimeline(summary({ overview: zero, apps: [], days: [], hours: [], segments: null }))).toBe(false);
  });
});
