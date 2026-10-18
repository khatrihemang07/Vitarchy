import { sharePercent, timelineFigures } from "./timelineFigures";

describe("sharePercent", () => {
  it("rounds to a whole percent", () => {
    expect(sharePercent(1, 3)).toBe(33);
  });
  it("is null when there is nothing to take a share of", () => {
    expect(sharePercent(5, 0)).toBeNull();
  });
  it("never exceeds 100", () => {
    expect(sharePercent(9, 3)).toBe(100);
  });
});

describe("timelineFigures", () => {
  it("takes shares of covered time, not of oldest-to-newest, so gaps count for nothing", () => {
    const figures = timelineFigures({
      segment_count: 4,
      oldest: "2026-04-01T00:00:00Z",
      newest: "2026-04-03T00:00:00Z",
      covered_secs: 36000,
      away_secs: 3600,
      unknown_secs: 18000,
    });
    expect(figures).toEqual({ awayShare: 10, unknownShare: 50 });
  });
  it("has no shares when nothing is covered", () => {
    expect(timelineFigures({ segment_count: 0, covered_secs: 0, away_secs: 0, unknown_secs: 0 })).toEqual({
      awayShare: null,
      unknownShare: null,
    });
  });
});
