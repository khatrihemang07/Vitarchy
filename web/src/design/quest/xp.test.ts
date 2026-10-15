import { xpProgress } from "./xp";

describe("xpProgress", () => {
  it("computes a whole-number percentage", () => {
    expect(xpProgress(2340, 3000)).toEqual({ value: 2340, max: 3000, percent: 78 });
  });

  it("clamps overflow to the maximum", () => {
    expect(xpProgress(5000, 3000)).toEqual({ value: 3000, max: 3000, percent: 100 });
  });

  it("clamps negative values to zero", () => {
    expect(xpProgress(-20, 100)).toEqual({ value: 0, max: 100, percent: 0 });
  });

  it("treats a non-positive or non-finite max as an empty bar", () => {
    expect(xpProgress(10, 0).percent).toBe(0);
    expect(xpProgress(10, Number.NaN).percent).toBe(0);
  });
});
