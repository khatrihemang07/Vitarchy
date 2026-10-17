import { hasFilters, parseReportSearch, rangeOf, toSearchParams } from "./reportSearch";

describe("parseReportSearch", () => {
  it("defaults to the current week grouped by description, per Day", () => {
    expect(parseReportSearch({})).toEqual({
      kind: "week", project: [], tag: [], client: [], q: "", group1: "description", mode: "per_day",
    });
  });

  it("reads every field", () => {
    const parsed = parseReportSearch({
      kind: "month", date: "2026-10-16", project: [3, 4], tag: ["deep"], client: [9], q: "stand",
      group1: "project", group2: "description", mode: "per_entry",
    });
    expect(parsed).toMatchObject({ kind: "month", date: "2026-10-16", project: [3, 4], tag: ["deep"], client: [9], q: "stand", group1: "project", group2: "description", mode: "per_entry" });
  });

  it("falls back for unknown kinds, groupings and modes", () => {
    const parsed = parseReportSearch({ kind: "year", group1: "x", group2: "y", mode: "z" });
    expect(parsed).toMatchObject({ kind: "week", group1: "description", mode: "per_day" });
    expect(parsed.group2).toBeUndefined();
  });

  it("drops a second level equal to the first", () => {
    expect(parseReportSearch({ group1: "tag", group2: "tag" }).group2).toBeUndefined();
  });

  it("ignores impossible dates", () => {
    expect(parseReportSearch({ date: "2026-02-30" }).date).toBeUndefined();
    expect(parseReportSearch({ date: "soon" }).date).toBeUndefined();
  });

  it("accepts a single value where a list is expected and drops junk ids", () => {
    const parsed = parseReportSearch({ project: 5, client: ["x", -1, 2.5, 7], tag: "a" });
    expect(parsed.project).toEqual([5]);
    expect(parsed.client).toEqual([7]);
    expect(parsed.tag).toEqual(["a"]);
  });

  it("keeps numeric-looking text as text (the router parses 2026 into a number)", () => {
    const parsed = parseReportSearch({ q: 2026, tag: [2026] });
    expect(parsed.q).toBe("2026");
    expect(parsed.tag).toEqual(["2026"]);
  });

  it("repairs a custom stretch: swaps reversed ends, caps the length, needs both ends", () => {
    expect(parseReportSearch({ kind: "custom", from: "2026-10-12", to: "2026-10-05" })).toMatchObject({ from: "2026-10-05", to: "2026-10-12" });
    expect(parseReportSearch({ kind: "custom", from: "2024-01-01", to: "2026-01-01" }).to).toBe("2024-12-31");
    expect(parseReportSearch({ kind: "custom", from: "2026-10-05" }).kind).toBe("week");
  });

  it("does not carry from and to for other kinds", () => {
    const parsed = parseReportSearch({ kind: "day", from: "2026-10-05", to: "2026-10-06" });
    expect(parsed.from).toBeUndefined();
  });
});

describe("parseReportSearch on odd URLs", () => {
  it("treats empty and missing lists the same", () => {
    const parsed = parseReportSearch({ project: [], tag: [], client: undefined });
    expect(parsed).toMatchObject({ project: [], tag: [], client: [] });
  });

  it("drops duplicate ids and names", () => {
    const parsed = parseReportSearch({ project: [1, 1, 2, 1], client: [4, 4], tag: ["a", "b", "a"] });
    expect(parsed).toMatchObject({ project: [1, 2], client: [4], tag: ["a", "b"] });
  });

  it("drops an empty tag, alone or among others", () => {
    expect(parseReportSearch({ tag: "" }).tag).toEqual([]);
    expect(parseReportSearch({ tag: ["", "a"] }).tag).toEqual(["a"]);
  });

  it("refuses negative, zero, fractional and non-numeric ids", () => {
    expect(parseReportSearch({ project: [-3] }).project).toEqual([]);
    expect(parseReportSearch({ project: [0, 1.5, "7", null, 8] }).project).toEqual([8]);
  });

  it("caps the search text at 200 characters", () => {
    expect(parseReportSearch({ q: "x".repeat(250) }).q).toHaveLength(200);
    expect(parseReportSearch({ q: "x".repeat(200) }).q).toHaveLength(200);
  });

  it("reads a search that is not text as none", () => {
    expect(parseReportSearch({ q: ["a"] }).q).toBe("");
    expect(parseReportSearch({ q: undefined }).q).toBe("");
  });

  it("leaves a blank search for the query to trim, but does not call it a filter", () => {
    const parsed = parseReportSearch({ q: "   " });
    expect(parsed.q).toBe("   ");
    expect(toSearchParams({ ...parsed, q: "" })).toEqual({});
  });

  it("ignores from and to unless the kind is custom", () => {
    expect(parseReportSearch({ kind: "week", from: "2026-10-01", to: "2026-10-09" })).not.toHaveProperty("from");
  });
});

describe("toSearchParams", () => {
  it("leaves defaults out so the URL stays short", () => {
    expect(toSearchParams(parseReportSearch({}))).toEqual({});
  });

  it("round-trips a full state", () => {
    const state = parseReportSearch({ kind: "custom", from: "2026-10-01", to: "2026-10-09", project: [1], tag: ["a"], q: "x", group1: "tag", group2: "project", mode: "per_entry" });
    expect(parseReportSearch(toSearchParams(state))).toEqual(state);
  });
});

describe("rangeOf", () => {
  it("anchors a week on the date, or today", () => {
    expect(rangeOf(parseReportSearch({ date: "2026-10-16" }), "2026-01-01", "monday")).toEqual({ from: "2026-10-12", to: "2026-10-18" });
    expect(rangeOf(parseReportSearch({}), "2026-10-16", "sunday")).toEqual({ from: "2026-10-11", to: "2026-10-17" });
  });

  it("uses a custom stretch as given", () => {
    expect(rangeOf(parseReportSearch({ kind: "custom", from: "2026-10-01", to: "2026-10-09" }), "2026-10-16", "monday")).toEqual({ from: "2026-10-01", to: "2026-10-09" });
  });
});

describe("hasFilters", () => {
  it("is true when a project, tag or client is chosen", () => {
    expect(hasFilters(parseReportSearch({}))).toBe(false);
    expect(hasFilters(parseReportSearch({ tag: ["a"] }))).toBe(true);
    expect(hasFilters(parseReportSearch({ q: "only text" }))).toBe(false);
  });
});
