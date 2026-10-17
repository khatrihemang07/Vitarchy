import { changeKind, fitRange, jumpToToday, periodFor, periodLabel, shiftPeriod, todayOf } from "./period";

describe("periodFor", () => {
  it("is the single Day for day", () => {
    expect(periodFor("day", "2026-10-16", "monday")).toEqual({ from: "2026-10-16", to: "2026-10-16" });
  });

  it("starts the week on the chosen week start", () => {
    // 2026-10-16 is a Friday.
    expect(periodFor("week", "2026-10-16", "monday")).toEqual({ from: "2026-10-12", to: "2026-10-18" });
    expect(periodFor("week", "2026-10-16", "sunday")).toEqual({ from: "2026-10-11", to: "2026-10-17" });
  });

  it("keeps a week start Day inside its own week", () => {
    expect(periodFor("week", "2026-10-12", "monday")).toEqual({ from: "2026-10-12", to: "2026-10-18" });
    expect(periodFor("week", "2026-10-18", "monday")).toEqual({ from: "2026-10-12", to: "2026-10-18" });
    expect(periodFor("week", "2026-10-11", "sunday")).toEqual({ from: "2026-10-11", to: "2026-10-17" });
  });

  it("covers the whole calendar month, leap February included", () => {
    expect(periodFor("month", "2026-10-16", "monday")).toEqual({ from: "2026-10-01", to: "2026-10-31" });
    expect(periodFor("month", "2028-02-10", "monday")).toEqual({ from: "2028-02-01", to: "2028-02-29" });
  });
});

describe("shiftPeriod", () => {
  it("moves a Day, a week and a month by one of themselves", () => {
    expect(shiftPeriod("day", { from: "2026-10-16", to: "2026-10-16" }, 1)).toEqual({ from: "2026-10-17", to: "2026-10-17" });
    expect(shiftPeriod("week", { from: "2026-10-12", to: "2026-10-18" }, -1)).toEqual({ from: "2026-10-05", to: "2026-10-11" });
    expect(shiftPeriod("month", { from: "2026-10-01", to: "2026-10-31" }, -1)).toEqual({ from: "2026-09-01", to: "2026-09-30" });
    expect(shiftPeriod("month", { from: "2026-12-01", to: "2026-12-31" }, 1)).toEqual({ from: "2027-01-01", to: "2027-01-31" });
  });

  it("moves a custom stretch by its own length", () => {
    expect(shiftPeriod("custom", { from: "2026-10-10", to: "2026-10-12" }, 1)).toEqual({ from: "2026-10-13", to: "2026-10-15" });
    expect(shiftPeriod("custom", { from: "2026-10-10", to: "2026-10-12" }, -1)).toEqual({ from: "2026-10-07", to: "2026-10-09" });
  });
});

describe("jumpToToday", () => {
  it("lands on the Period containing today", () => {
    expect(jumpToToday("week", { from: "2026-09-01", to: "2026-09-07" }, "2026-10-16", "monday")).toEqual({ from: "2026-10-12", to: "2026-10-18" });
  });

  it("ends a custom stretch today, keeping its length", () => {
    expect(jumpToToday("custom", { from: "2026-09-01", to: "2026-09-03" }, "2026-10-16", "monday")).toEqual({ from: "2026-10-14", to: "2026-10-16" });
  });
});

describe("changeKind", () => {
  const week = { from: "2026-10-12", to: "2026-10-18" };

  it("prefers today when the current Period contains it", () => {
    expect(changeKind("day", week, "2026-10-16", "monday")).toEqual({ from: "2026-10-16", to: "2026-10-16" });
  });

  it("otherwise starts from the Period's first Day", () => {
    expect(changeKind("month", week, "2026-12-01", "monday")).toEqual({ from: "2026-10-01", to: "2026-10-31" });
  });

  it("keeps the current stretch when going custom", () => {
    expect(changeKind("custom", week, "2026-10-16", "monday")).toEqual(week);
  });
});

describe("periodLabel", () => {
  it("names a Day with its weekday", () => {
    expect(periodLabel("day", { from: "2026-10-16", to: "2026-10-16" })).toBe("Fri 16 Oct 2026");
  });

  it("writes a week inside one month compactly", () => {
    expect(periodLabel("week", { from: "2026-10-12", to: "2026-10-18" })).toBe("12–18 Oct 2026");
  });

  it("spells out both months or years when the stretch crosses them", () => {
    expect(periodLabel("week", { from: "2026-09-28", to: "2026-10-04" })).toBe("28 Sep – 4 Oct 2026");
    expect(periodLabel("custom", { from: "2026-12-28", to: "2027-01-03" })).toBe("28 Dec 2026 – 3 Jan 2027");
  });

  it("names a month in full", () => {
    expect(periodLabel("month", { from: "2026-10-01", to: "2026-10-31" })).toBe("October 2026");
  });

  it("labels a one-Day custom stretch as a Day", () => {
    expect(periodLabel("custom", { from: "2026-10-16", to: "2026-10-16" })).toBe("Fri 16 Oct 2026");
  });
});

describe("todayOf", () => {
  it("uses the local calendar Day", () => {
    expect(todayOf(new Date(2026, 9, 16, 23, 59))).toBe("2026-10-16");
    expect(todayOf(new Date(2026, 0, 5, 0, 1))).toBe("2026-01-05");
  });
});

describe("fitRange", () => {
  it("puts the ends in order", () => {
    expect(fitRange({ from: "2026-10-12", to: "2026-10-05" })).toEqual({ from: "2026-10-05", to: "2026-10-12" });
  });

  it("caps the length at what the server accepts", () => {
    expect(fitRange({ from: "2024-01-01", to: "2026-01-01" })).toEqual({ from: "2024-01-01", to: "2024-12-31" });
  });
});
