import { clearFilters, hasAnyFilter, withFacet, withGroups, withRange } from "./reportActions";
import { parseReportSearch } from "./reportSearch";

const base = parseReportSearch({ project: [1], tag: ["a"], q: "x", group1: "project", group2: "tag" });

describe("withRange", () => {
  it("keeps one anchor Day for a week, month or Day", () => {
    const next = withRange(base, "week", { from: "2026-10-12", to: "2026-10-18" });
    expect(next).toMatchObject({ kind: "week", date: "2026-10-12" });
    expect(next.from).toBeUndefined();
  });

  it("keeps both ends for a custom stretch", () => {
    const next = withRange(base, "custom", { from: "2026-10-01", to: "2026-10-09" });
    expect(next).toMatchObject({ kind: "custom", from: "2026-10-01", to: "2026-10-09" });
    expect(next.date).toBeUndefined();
  });
});

describe("withFacet", () => {
  it("turns text values into ids for projects and clients", () => {
    expect(withFacet(base, "project", ["2", "3"]).project).toEqual([2, 3]);
    expect(withFacet(base, "client", ["7"]).client).toEqual([7]);
  });

  it("keeps tag names as text", () => {
    expect(withFacet(base, "tag", ["x", "y"]).tag).toEqual(["x", "y"]);
  });
});

describe("withGroups", () => {
  it("drops a second level that equals the new first level", () => {
    expect(withGroups(base, "tag", "tag").group2).toBeUndefined();
    expect(withGroups(base, "tag", undefined)).toMatchObject({ group1: "tag" });
  });

  it("sets both levels", () => {
    expect(withGroups(base, "description", "project")).toMatchObject({ group1: "description", group2: "project" });
  });
});

describe("clearFilters", () => {
  it("empties projects, tags, clients and the search, but keeps Period and grouping", () => {
    const cleared = clearFilters(base);
    expect(cleared).toMatchObject({ project: [], tag: [], client: [], q: "", group1: "project", group2: "tag" });
  });
});

describe("hasAnyFilter", () => {
  it("counts the description search too", () => {
    expect(hasAnyFilter(parseReportSearch({ q: "x" }))).toBe(true);
    expect(hasAnyFilter(parseReportSearch({}))).toBe(false);
  });
});
