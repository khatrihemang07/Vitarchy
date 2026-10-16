import { fractionOf, formatShare, formatSpan, toHours } from "./duration";

describe("formatSpan", () => {
  it.each([
    [0, "0 s"],
    [30, "30 s"],
    [59.4, "59 s"],
    [60, "1 min"],
    [45 * 60, "45 min"],
    [3600, "1 h"],
    [85 * 60, "1 h 25 min"],
    [76 * 3600, "76 h"],
  ])("writes %s seconds as %s", (seconds, text) => {
    expect(formatSpan(seconds)).toBe(text);
  });
});

describe("formatShare", () => {
  it("rounds to whole percent and flags a sliver", () => {
    expect(formatShare(1, 3)).toBe("33%");
    expect(formatShare(1, 1000)).toBe("<1%");
  });

  it("is 0% when there is nothing", () => {
    expect(formatShare(0, 100)).toBe("0%");
    expect(formatShare(5, 0)).toBe("0%");
  });
});

describe("toHours", () => {
  it("keeps one decimal", () => {
    expect(toHours(5400)).toBe(1.5);
    expect(toHours(3700)).toBe(1);
  });
});

describe("fractionOf", () => {
  it("is the part of the longest, from 0 to 1", () => {
    expect(fractionOf(30, 120)).toBe(0.25);
  });

  it("is 0, never NaN, when nothing was recorded", () => {
    expect(fractionOf(0, 0)).toBe(0);
  });
});
