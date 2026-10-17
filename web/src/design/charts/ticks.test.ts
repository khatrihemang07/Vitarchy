import { niceTicks } from "./ticks";

describe("niceTicks", () => {
  it("steps by whole hours for a long Day", () => {
    expect(niceTicks(8 * 3600 + 1200)).toEqual([0, 3 * 3600, 6 * 3600, 9 * 3600]);
  });

  it("steps by minutes for short bars", () => {
    expect(niceTicks(25 * 60)).toEqual([0, 600, 1200, 1800]);
  });

  it("covers the largest bar with at most four steps", () => {
    for (const max of [1, 59, 900, 5000, 40_000, 86_400]) {
      const ticks = niceTicks(max);
      expect(ticks.at(-1)).toBeGreaterThanOrEqual(max);
      expect(ticks.length).toBeLessThanOrEqual(5);
    }
  });

  it("still draws an axis when there is no time", () => {
    expect(niceTicks(0)).toEqual([0, 60]);
  });
});
