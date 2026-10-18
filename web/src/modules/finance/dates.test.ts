import { formatDate, formatDateAndTime, formatLastUsed, formatPeriod } from "./dates";

describe("formatDate", () => {
  it("writes a printed date without shifting it across time zones", () => {
    expect(formatDate("2026-04-01")).toBe("1 Apr 2026");
    expect(formatDate("2026-12-31")).toBe("31 Dec 2026");
  });
  it("leaves something that is not a date as it is", () => {
    expect(formatDate("soon")).toBe("soon");
  });
});

describe("formatPeriod", () => {
  it("names both ends, one day, or whichever end is known", () => {
    expect(formatPeriod({ start: "2026-04-01", end: "2026-04-30" })).toBe("1 Apr 2026 to 30 Apr 2026");
    expect(formatPeriod({ start: "2026-04-01", end: "2026-04-01" })).toBe("1 Apr 2026");
    expect(formatPeriod({ start: "2026-04-01" })).toBe("from 1 Apr 2026");
    expect(formatPeriod({ end: "2026-04-30" })).toBe("to 30 Apr 2026");
    expect(formatPeriod({})).toBe("Period not printed");
  });
});

describe("formatDateAndTime", () => {
  it("puts the time next to the date, to the minute", () => {
    expect(formatDateAndTime("2026-08-18", "11:20:45")).toBe("18 Aug 2026 · 11:20");
  });
  it("shows only the date when no time was printed", () => {
    expect(formatDateAndTime("2026-08-18", null)).toBe("18 Aug 2026");
    expect(formatDateAndTime("2026-08-18", undefined)).toBe("18 Aug 2026");
  });
});

describe("formatLastUsed", () => {
  it("says when a password last opened a file, or that it never did", () => {
    expect(formatLastUsed("2026-05-03T08:00:00Z")).toBe("Last used 3 May 2026");
    expect(formatLastUsed(null)).toBe("Never used");
  });
});
