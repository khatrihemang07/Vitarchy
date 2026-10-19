import { COMPARE, compareNote, compareOf, defaultCompare } from "./compareMode";
import { group, report } from "./testReport";
import { USUAL_LABEL, USUAL_PHRASE } from "../usualWords";
import { parseReportSearch, toSearchParams } from "./reportSearch";
import { withRange } from "@/lib/period/periodSearch";

describe("defaultCompare", () => {
  it("compares a Day with the last 4 weeks and every other kind with the Previous Period", () => {
    expect(defaultCompare("day")).toBe("usual");
    expect(defaultCompare("week")).toBe("previous");
    expect(defaultCompare("month")).toBe("previous");
    expect(defaultCompare("custom")).toBe("previous");
  });
});

describe("compare in the URL", () => {
  it("falls back to the default for the kind when the URL is silent or wrong", () => {
    expect(compareOf(parseReportSearch({ kind: "day" }))).toBe("usual");
    expect(compareOf(parseReportSearch({ kind: "week" }))).toBe("previous");
    expect(compareOf(parseReportSearch({ kind: "week", compare: "nonsense" }))).toBe("previous");
  });

  it("reads a valid choice, which wins over the default", () => {
    expect(compareOf(parseReportSearch({ kind: "week", compare: "usual" }))).toBe("usual");
    expect(compareOf(parseReportSearch({ kind: "day", compare: "previous" }))).toBe("previous");
  });

  it("writes only a choice that differs from the default of the kind", () => {
    expect(toSearchParams(parseReportSearch({ kind: "week", compare: "usual" }))).toMatchObject({ compare: "usual" });
    expect(toSearchParams(parseReportSearch({ kind: "week", compare: "previous" }))).not.toHaveProperty("compare");
    expect(toSearchParams(parseReportSearch({ kind: "day", compare: "usual" }))).not.toHaveProperty("compare");
    expect(toSearchParams(parseReportSearch({ kind: "day", compare: "previous" }))).toMatchObject({ compare: "previous" });
  });

  it("is kept when the Period changes", () => {
    const search = parseReportSearch({ kind: "day", compare: "previous" });
    expect(withRange(search, "week", { from: "2026-10-12", to: "2026-10-18" }).compare).toBe("previous");
  });
});

const data = report([]);
const withUsual = { ...data, overview: { ...data.overview, usual_days: 18, vs_usual: { total_s: -5, entries: 1, days_tracked: 2, avg_per_day_s: 3 } } };
const withHistory = (days: number, usualDays = 18) => ({
  ...withUsual,
  period: { ...withUsual.period, usual_available_days: days },
  overview: { ...withUsual.overview, usual_days: usualDays },
});

describe("COMPARE words", () => {
  it("names the Usual once, from the shared wording", () => {
    expect(COMPARE.usual.label).toBe(USUAL_LABEL);
    expect(COMPARE.usual.phrase).toBe(USUAL_PHRASE);
    expect(COMPARE.previous.label).toBe("Previous Period");
  });
});

describe("COMPARE.*.group", () => {
  const usual = { days: 9, per_day: { avg_s: 1, min_s: 2, max_s: 3 }, per_entry: { avg_s: 4, min_s: 5, max_s: 6 } };
  const change = { days: 0, entries: 0, total_s: 7, per_day: { avg_s: 8, min_s: 0, max_s: 0 }, per_entry: { avg_s: 0, min_s: 0, max_s: 0 } };
  const g = group("a", { vs_previous: { is_new: false, change }, vs_usual: { is_new: true, usual } });

  it("reads the Previous Period's comparison, with no Usual", () => {
    expect(COMPARE.previous.group(g)).toEqual({ change, isNew: false, usual: null });
  });

  it("reads the Usual's comparison", () => {
    expect(COMPARE.usual.group(g)).toEqual({ change: null, isNew: true, usual });
  });
});

describe("COMPARE.*.dayTotalChange", () => {
  const change = { days: 0, entries: 0, total_s: 7, per_day: { avg_s: 8, min_s: 0, max_s: 0 }, per_entry: { avg_s: 0, min_s: 0, max_s: 0 } };

  it("is yesterday's total difference against the Previous Period and the average per Day against the Usual", () => {
    expect(COMPARE.previous.dayTotalChange(change)).toBe(7);
    expect(COMPARE.usual.dayTotalChange(change)).toBe(8);
  });
});

describe("COMPARE.*.overview", () => {
  it("picks the changes of the chosen comparison", () => {
    expect(COMPARE.previous.overview(withUsual.overview).total_s).toBe(600);
    expect(COMPARE.usual.overview(withUsual.overview).total_s).toBe(-5);
  });
});

describe("compareNote", () => {
  it("names the last 4 weeks with their dates and how many Days had entries", () => {
    expect(compareNote(withUsual, "usual")).toBe("Compared with the last 4 weeks, 14 Sep – 11 Oct 2026 (18 Days with entries)");
  });

  it("says 'Day' for exactly one Day with entries and 'no Days' for none", () => {
    expect(compareNote(withHistory(28, 1), "usual")).toContain("(1 Day with entries)");
    expect(compareNote(data, "usual")).toContain("(no Days with entries)");
  });

  it("names the Previous Period with its dates", () => {
    expect(compareNote(data, "previous")).toBe("Compared with the Previous Period, 5–11 Oct 2026");
  });

  it("names the last Day with entries when the Previous Period was skipped", () => {
    const skipped = { ...data, period: { ...data.period, kind: "day" as const, previous_from: "2026-10-09", previous_to: "2026-10-09", previous_skipped: true } };
    expect(compareNote(skipped, "previous")).toBe("Compared with the last Day with entries, Fri 9 Oct 2026");
  });

  it("names the kind of Period that was skipped back to", () => {
    const skipped = (kind: "week" | "month" | "custom") => ({ ...data, period: { ...data.period, kind, previous_skipped: true } });
    expect(compareNote(skipped("week"), "previous")).toBe("Compared with the last week with entries, 5–11 Oct 2026");
    expect(compareNote(skipped("month"), "previous")).toMatch(/^Compared with the last month with entries, /);
    expect(compareNote(skipped("custom"), "previous")).toMatch(/^Compared with the last same-length stretch with entries, /);
  });

  it("says how short the history is when it does not cover the whole range", () => {
    expect(compareNote(withHistory(10), "usual")).toBe("Compared with the last 4 weeks, 14 Sep – 11 Oct 2026 (18 Days with entries). Only 10 Days of history so far");
    expect(compareNote(withHistory(1), "usual")).toContain("Only 1 Day of history so far");
  });

  it("says there is no history yet when none of the range is available", () => {
    expect(compareNote(withHistory(0, 0), "usual")).toContain("No history yet");
  });

  it("leaves the history out of the Previous Period's note", () => {
    expect(compareNote(withHistory(10), "previous")).not.toContain("history");
  });
});
