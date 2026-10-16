import {
  canGoNext,
  isIsoDate,
  labelPeriod,
  localIsoDate,
  previousPeriod,
  selectionFor,
  shiftPeriod,
  spanDays,
  switchKind,
  type PeriodSelection,
} from "./period";

const week = (from: string, to: string): PeriodSelection => ({ kind: "week", from, to });

describe("selectionFor", () => {
  it("makes a one-day period from its anchor", () => {
    expect(selectionFor("day", "2026-10-14", "monday")).toEqual({ kind: "day", from: "2026-10-14", to: "2026-10-14" });
  });

  it("starts a week on the chosen weekday", () => {
    // 2026-10-14 is a Wednesday.
    expect(selectionFor("week", "2026-10-14", "monday")).toEqual(week("2026-10-12", "2026-10-18"));
    expect(selectionFor("week", "2026-10-14", "sunday")).toEqual(week("2026-10-11", "2026-10-17"));
  });

  it("keeps a week whose first day is the anchor", () => {
    expect(selectionFor("week", "2026-10-11", "sunday")).toEqual(week("2026-10-11", "2026-10-17"));
  });

  it("spans a whole calendar month, including leap February", () => {
    expect(selectionFor("month", "2026-10-14", "monday")).toMatchObject({ from: "2026-10-01", to: "2026-10-31" });
    expect(selectionFor("month", "2028-02-10", "monday")).toMatchObject({ from: "2028-02-01", to: "2028-02-29" });
  });

  it("makes a custom range of the anchor day until it is edited", () => {
    expect(selectionFor("custom", "2026-10-14", "monday")).toEqual({ kind: "custom", from: "2026-10-14", to: "2026-10-14" });
  });
});

describe("switchKind", () => {
  it("re-anchors on the start of the current period", () => {
    expect(switchKind(week("2026-10-12", "2026-10-18"), "month", "monday")).toMatchObject({ from: "2026-10-01", to: "2026-10-31" });
  });

  it("keeps the current range when switching to custom", () => {
    expect(switchKind(week("2026-10-12", "2026-10-18"), "custom", "monday")).toEqual({
      kind: "custom",
      from: "2026-10-12",
      to: "2026-10-18",
    });
  });
});

describe("shiftPeriod and previousPeriod", () => {
  it("moves a day, a week and a month by one unit", () => {
    expect(shiftPeriod(selectionFor("day", "2026-03-01", "monday"), -1)).toMatchObject({ from: "2026-02-28" });
    expect(shiftPeriod(week("2026-10-12", "2026-10-18"), 1)).toEqual(week("2026-10-19", "2026-10-25"));
    expect(shiftPeriod(selectionFor("month", "2026-03-10", "monday"), -1)).toMatchObject({ from: "2026-02-01", to: "2026-02-28" });
    expect(shiftPeriod(selectionFor("month", "2026-12-10", "monday"), 1)).toMatchObject({ from: "2027-01-01", to: "2027-01-31" });
  });

  it("moves a custom range by its own length", () => {
    const custom: PeriodSelection = { kind: "custom", from: "2026-10-10", to: "2026-10-12" };
    expect(shiftPeriod(custom, -1)).toEqual({ kind: "custom", from: "2026-10-07", to: "2026-10-09" });
  });

  it("crosses a year end", () => {
    expect(shiftPeriod(week("2026-12-28", "2027-01-03"), 1)).toEqual(week("2027-01-04", "2027-01-10"));
  });

  it("names the period right before as the previous one", () => {
    expect(previousPeriod(week("2026-10-12", "2026-10-18"))).toEqual(week("2026-10-05", "2026-10-11"));
  });
});

describe("canGoNext", () => {
  it("allows moving forward only to a period that has started", () => {
    const current = week("2026-10-12", "2026-10-18");
    expect(canGoNext(current, "2026-10-18")).toBe(false);
    expect(canGoNext(current, "2026-10-19")).toBe(true);
  });
});

describe("canGoNext for a custom range", () => {
  const custom: PeriodSelection = { kind: "custom", from: "2026-10-10", to: "2026-10-12" };
  it("stops when the next range would end after today", () => {
    expect(canGoNext(custom, "2026-10-14")).toBe(false);
    expect(canGoNext(custom, "2026-10-15")).toBe(true);
  });
});

describe("spanDays", () => {
  it("counts both ends", () => {
    expect(spanDays("2026-10-10", "2026-10-10")).toBe(1);
    expect(spanDays("2026-10-10", "2026-10-12")).toBe(3);
  });
});

describe("labelPeriod", () => {
  it("names a day, a month and a range", () => {
    expect(labelPeriod(selectionFor("day", "2026-10-14", "monday"), "en-US")).toBe("Oct 14, 2026");
    expect(labelPeriod(selectionFor("month", "2026-10-14", "monday"), "en-US")).toBe("October 2026");
    expect(labelPeriod(week("2026-10-12", "2026-10-18"), "en-US")).toMatch(/Oct 12.*18, 2026/);
  });
});

describe("dates", () => {
  it("formats the local calendar date, not the UTC one", () => {
    expect(localIsoDate(new Date(2026, 9, 5, 23, 59))).toBe("2026-10-05");
  });

  it("accepts only real YYYY-MM-DD dates", () => {
    expect(isIsoDate("2026-02-28")).toBe(true);
    expect(isIsoDate("2026-02-30")).toBe(false);
    expect(isIsoDate("2026-2-3")).toBe(false);
    expect(isIsoDate("tomorrow")).toBe(false);
  });
});
