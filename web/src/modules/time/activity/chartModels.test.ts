import { boundsPoints, trendChart } from "./chartModels";

describe("boundsPoints", () => {
  it("has a point for every Day, empty for Days without Activity", () => {
    const points = boundsPoints("2026-05-01", "2026-05-03", [{ date: "2026-05-02", first_minute: 540, last_minute: 1020 }]);
    expect(points).toEqual([
      { date: "2026-05-01", first: null, last: null },
      { date: "2026-05-02", first: 540, last: 1020 },
      { date: "2026-05-03", first: null, last: null },
    ]);
  });
});

describe("trendChart", () => {
  const trend = {
    weeks: [
      { start: "2026-05-04", days: 7 },
      { start: "2026-05-11", days: 2 },
    ],
    series: [
      { app: "Code", seconds: [7200, 3600] },
      { app: "Spotify", seconds: [1800, 0] },
    ],
    other: [0, 900],
  };

  it("gives hours per week under a key per series and colours them in order", () => {
    const { rows, series } = trendChart(trend);
    expect(rows[0]).toMatchObject({ start: "2026-05-04", days: 7, s0: 2, s1: 0.5, other: 0 });
    expect(rows[1]).toMatchObject({ days: 2, s0: 1, s1: 0, other: 0.3 });
    expect(series.map((s) => [s.label, s.color])).toEqual([
      ["Code", "var(--chart-1)"],
      ["Spotify", "var(--chart-2)"],
      ["Other apps", "var(--chart-other)"],
    ]);
  });

  it("leaves out Other when nothing folded into it", () => {
    const { rows, series } = trendChart({ ...trend, other: [0, 0] });
    expect(series.map((s) => s.label)).toEqual(["Code", "Spotify"]);
    expect(rows[0]).not.toHaveProperty("other");
  });
});
