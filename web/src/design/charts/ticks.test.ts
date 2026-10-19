import { decimalTicks, niceTicks } from "./ticks";

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

describe("decimalTicks", () => {
  it("starts at 0 and steps by 1, 2 or 5 times a power of ten", () => {
    expect(decimalTicks(864_000)).toEqual([0, 500_000, 1_000_000]);
    expect(decimalTicks(260_000)).toEqual([0, 100_000, 200_000, 300_000]);
    expect(decimalTicks(7)).toEqual([0, 2, 4, 6, 8]);
  });

  it("covers the largest bar with at most four steps for any size", () => {
    for (const max of [1, 99, 1234, 86_400, 864_000, 12_345_678, 9_999_999_999]) {
      const ticks = decimalTicks(max);
      expect(ticks[0]).toBe(0);
      expect(ticks.at(-1)).toBeGreaterThanOrEqual(max);
      expect(ticks.length).toBeLessThanOrEqual(6);
    }
  });

  it("still draws an axis when there is nothing", () => {
    expect(decimalTicks(0)).toEqual([0, 1]);
  });
});
