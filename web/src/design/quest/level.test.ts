import { normalizeLevel } from "./level";

describe("normalizeLevel", () => {
  it.each([
    [12, 12],
    [0, 0],
    [3.9, 3],
    [-2.7, 0],
    [-0.5, 0],
  ])("%d becomes %d", (input, expected) => {
    expect(normalizeLevel(input)).toBe(expected);
  });

  it("falls back to 0 for non-finite input", () => {
    expect(normalizeLevel(Number.NaN)).toBe(0);
    expect(normalizeLevel(Number.POSITIVE_INFINITY)).toBe(0);
  });
});
