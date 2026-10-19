import { addDays, changeKind, containsDay, dayCount, fitRange, jumpToToday, periodFor, periodLabel, shiftPeriod, todayOf } from "./period";

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

describe("the test time zone", () => {
  it("is pinned to America/Los_Angeles, where October is UTC-7", () => {
    expect(new Date("2026-10-17T12:00:00Z").getTimezoneOffset()).toBe(420);
    expect(new Date("2026-12-17T12:00:00Z").getTimezoneOffset()).toBe(480);
  });
});

describe("todayOf", () => {
  it("is the local Day even when the UTC date is already the next one", () => {
    // 05:00Z is 22:00 on the 16th in Los Angeles.
    expect(todayOf(new Date("2026-10-17T05:00:00Z"))).toBe("2026-10-16");
  });

  it("turns over at local midnight, not at UTC midnight", () => {
    expect(todayOf(new Date("2026-10-17T06:59:59Z"))).toBe("2026-10-16");
    expect(todayOf(new Date("2026-10-17T07:00:00Z"))).toBe("2026-10-17");
  });

  it("pads single-digit months and Days", () => {
    expect(todayOf(new Date("2026-01-05T09:01:00Z"))).toBe("2026-01-05");
  });

  it("keeps the same Day through the repeated hour when the clocks go back", () => {
    // 1 Nov 2026: 01:30 happens twice (PDT at 08:30Z, PST at 09:30Z).
    expect(todayOf(new Date("2026-11-01T08:30:00Z"))).toBe("2026-11-01");
    expect(todayOf(new Date("2026-11-01T09:30:00Z"))).toBe("2026-11-01");
    // The 25-hour Day ends at 08:00Z on the 2nd.
    expect(todayOf(new Date("2026-11-02T07:59:00Z"))).toBe("2026-11-01");
    expect(todayOf(new Date("2026-11-02T08:00:00Z"))).toBe("2026-11-02");
  });
});

describe("addDays", () => {
  it("moves by calendar Days across a month, a year and a leap Day", () => {
    expect(addDays("2026-10-31", 1)).toBe("2026-11-01");
    expect(addDays("2026-12-31", 1)).toBe("2027-01-01");
    expect(addDays("2028-02-28", 1)).toBe("2028-02-29");
    expect(addDays("2027-02-28", 1)).toBe("2027-03-01");
    expect(addDays("2026-03-01", -1)).toBe("2026-02-28");
  });

  it("is not shifted by a daylight-saving change", () => {
    expect(addDays("2026-10-31", 2)).toBe("2026-11-02");
    expect(addDays("2026-03-07", 2)).toBe("2026-03-09");
  });

  it("moves by zero and by many", () => {
    expect(addDays("2026-10-16", 0)).toBe("2026-10-16");
    expect(addDays("2026-01-01", 365)).toBe("2027-01-01");
  });
});

describe("dayCount", () => {
  it("counts both ends", () => {
    expect(dayCount({ from: "2026-10-16", to: "2026-10-16" })).toBe(1);
    expect(dayCount({ from: "2026-10-12", to: "2026-10-18" })).toBe(7);
  });

  it("counts a leap year as 366 and a leap February as 29", () => {
    expect(dayCount({ from: "2028-01-01", to: "2028-12-31" })).toBe(366);
    expect(dayCount({ from: "2028-02-01", to: "2028-02-29" })).toBe(29);
  });

  it("counts a Day that has 25 hours as one Day", () => {
    expect(dayCount({ from: "2026-10-31", to: "2026-11-02" })).toBe(3);
  });
});

describe("containsDay", () => {
  const range = { from: "2026-10-12", to: "2026-10-18" };

  it("includes both ends", () => {
    expect(containsDay(range, "2026-10-12")).toBe(true);
    expect(containsDay(range, "2026-10-18")).toBe(true);
    expect(containsDay(range, "2026-10-15")).toBe(true);
  });

  it("excludes the Days just outside", () => {
    expect(containsDay(range, "2026-10-11")).toBe(false);
    expect(containsDay(range, "2026-10-19")).toBe(false);
  });

  it("works across a year end", () => {
    const crossing = { from: "2026-12-30", to: "2027-01-02" };
    expect(containsDay(crossing, "2027-01-01")).toBe(true);
    expect(containsDay(crossing, "2026-12-29")).toBe(false);
  });
});

describe("year-crossing and leap-year navigation", () => {
  it("steps a Day, a week and a month back and forward over the new year", () => {
    expect(shiftPeriod("day", { from: "2026-01-01", to: "2026-01-01" }, -1)).toEqual({ from: "2025-12-31", to: "2025-12-31" });
    expect(shiftPeriod("day", { from: "2026-12-31", to: "2026-12-31" }, 1)).toEqual({ from: "2027-01-01", to: "2027-01-01" });
    expect(shiftPeriod("week", { from: "2026-01-05", to: "2026-01-11" }, -1)).toEqual({ from: "2025-12-29", to: "2026-01-04" });
    expect(shiftPeriod("week", { from: "2026-12-28", to: "2027-01-03" }, 1)).toEqual({ from: "2027-01-04", to: "2027-01-10" });
    expect(shiftPeriod("month", { from: "2026-01-01", to: "2026-01-31" }, -1)).toEqual({ from: "2025-12-01", to: "2025-12-31" });
    expect(shiftPeriod("custom", { from: "2026-12-30", to: "2027-01-02" }, 1)).toEqual({ from: "2027-01-03", to: "2027-01-06" });
    expect(shiftPeriod("custom", { from: "2026-01-02", to: "2026-01-04" }, -1)).toEqual({ from: "2025-12-30", to: "2026-01-01" });
  });

  it("steps a month over a leap and a non-leap February", () => {
    expect(shiftPeriod("month", { from: "2028-03-01", to: "2028-03-31" }, -1)).toEqual({ from: "2028-02-01", to: "2028-02-29" });
    expect(shiftPeriod("month", { from: "2027-03-01", to: "2027-03-31" }, -1)).toEqual({ from: "2027-02-01", to: "2027-02-28" });
    expect(shiftPeriod("month", { from: "2028-02-01", to: "2028-02-29" }, 1)).toEqual({ from: "2028-03-01", to: "2028-03-31" });
  });

  it("starts a Sunday week on the Sunday, across a year end", () => {
    // 2027-01-01 is a Friday.
    expect(periodFor("week", "2027-01-01", "sunday")).toEqual({ from: "2026-12-27", to: "2027-01-02" });
    expect(periodFor("week", "2027-01-01", "monday")).toEqual({ from: "2026-12-28", to: "2027-01-03" });
  });

  it("jumps a Sunday week to the week holding today", () => {
    // 2026-10-18 is a Sunday: it starts a Sunday week but ends a Monday one.
    expect(jumpToToday("week", { from: "2026-01-04", to: "2026-01-10" }, "2026-10-18", "sunday")).toEqual({ from: "2026-10-18", to: "2026-10-24" });
    expect(jumpToToday("week", { from: "2026-01-04", to: "2026-01-10" }, "2026-10-18", "monday")).toEqual({ from: "2026-10-12", to: "2026-10-18" });
  });
});

describe("fitRange", () => {
  it("accepts an exact 366-Day stretch and trims a 367-Day one", () => {
    expect(fitRange({ from: "2028-01-01", to: "2028-12-31" })).toEqual({ from: "2028-01-01", to: "2028-12-31" });
    expect(fitRange({ from: "2028-01-01", to: "2029-01-01" })).toEqual({ from: "2028-01-01", to: "2028-12-31" });
  });

  it("puts the ends in order", () => {
    expect(fitRange({ from: "2026-10-12", to: "2026-10-05" })).toEqual({ from: "2026-10-05", to: "2026-10-12" });
  });

  it("caps the length at what the server accepts", () => {
    expect(fitRange({ from: "2024-01-01", to: "2026-01-01" })).toEqual({ from: "2024-01-01", to: "2024-12-31" });
  });
});
