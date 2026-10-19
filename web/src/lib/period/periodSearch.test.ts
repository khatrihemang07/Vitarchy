import { parsePeriodSearch, periodParams, rangeOf, validDay, withRange } from "./periodSearch";

describe("validDay", () => {
  it("accepts real Days only", () => {
    expect(validDay("2026-02-28")).toBe("2026-02-28");
    expect(validDay("2026-02-30")).toBeUndefined();
    expect(validDay("soon")).toBeUndefined();
    expect(validDay(20260228)).toBeUndefined();
  });
});

describe("parsePeriodSearch", () => {
  it("falls back to the default kind for anything unusable", () => {
    expect(parsePeriodSearch({}, "month")).toEqual({ kind: "month" });
    expect(parsePeriodSearch({ kind: "year" }, "week")).toEqual({ kind: "week" });
  });
  it("keeps a real kind and Day", () => {
    expect(parsePeriodSearch({ kind: "day", date: "2026-10-16" }, "month")).toEqual({ kind: "day", date: "2026-10-16" });
  });
  it("repairs a custom stretch and needs both ends", () => {
    expect(parsePeriodSearch({ kind: "custom", from: "2026-10-12", to: "2026-10-05" }, "month")).toEqual({ kind: "custom", from: "2026-10-05", to: "2026-10-12" });
    expect(parsePeriodSearch({ kind: "custom", from: "2026-10-05" }, "month")).toEqual({ kind: "month" });
  });
});

describe("periodParams", () => {
  it("leaves out the default kind only", () => {
    expect(periodParams({ kind: "month" }, "month")).toEqual({});
    expect(periodParams({ kind: "week", date: "2026-10-16" }, "month")).toEqual({ kind: "week", date: "2026-10-16" });
  });
});

describe("rangeOf", () => {
  it("is the Period holding the named Day, or today", () => {
    expect(rangeOf({ kind: "month", date: "2026-10-16" }, "2026-01-01", "monday")).toEqual({ from: "2026-10-01", to: "2026-10-31" });
    expect(rangeOf({ kind: "month" }, "2026-02-10", "monday")).toEqual({ from: "2026-02-01", to: "2026-02-28" });
  });
  it("is a custom stretch as given", () => {
    expect(rangeOf({ kind: "custom", from: "2026-10-01", to: "2026-10-09" }, "2026-10-16", "monday")).toEqual({ from: "2026-10-01", to: "2026-10-09" });
  });
});

describe("withRange", () => {
  const base = { kind: "week" as const, date: "2026-10-01", q: "x" };
  it("keeps one anchor Day for a week, month or Day", () => {
    const next = withRange(base, "month", { from: "2026-10-01", to: "2026-10-31" });
    expect(next).toEqual({ kind: "month", date: "2026-10-01", q: "x" });
  });
  it("keeps both ends for a custom stretch", () => {
    const next = withRange(base, "custom", { from: "2026-10-01", to: "2026-10-09" });
    expect(next).toEqual({ kind: "custom", from: "2026-10-01", to: "2026-10-09", q: "x" });
  });
});
