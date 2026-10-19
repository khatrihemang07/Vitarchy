import { clearFilters, hasAnyFilter, withFacet, withGroups } from "./reportActions";
import { parseReportSearch } from "./reportSearch";

const base = parseReportSearch({ project: [1], tag: ["a"], q: "x", group1: "project", group2: "tag" });

describe("withFacet", () => {
  it("turns text values into ids for projects and clients", () => {
    expect(withFacet(base, "project", ["2", "3"]).project).toEqual([2, 3]);
    expect(withFacet(base, "client", ["7"]).client).toEqual([7]);
  });

  it("keeps tag names as text", () => {
    expect(withFacet(base, "tag", ["x", "y"]).tag).toEqual(["x", "y"]);
  });

  it("replaces only the chosen filter", () => {
    expect(withFacet(base, "client", ["7"])).toMatchObject({ project: [1], tag: ["a"], client: [7] });
    expect(withFacet(base, "project", [])).toMatchObject({ project: [], tag: ["a"] });
  });
});

describe("withGroups", () => {
  it("drops a second level that equals the new first level", () => {
    expect(withGroups(base, "tag", "tag").group2).toBeUndefined();
    expect(withGroups(base, "tag", "tag")).not.toHaveProperty("group2");
  });

  it("drops the old second level when none is given", () => {
    const next = withGroups(base, "tag", undefined);
    expect(next.group1).toBe("tag");
    expect(next).not.toHaveProperty("group2");
  });

  it("keeps the filters and Period while regrouping", () => {
    expect(withGroups(base, "description", undefined)).toMatchObject({ project: [1], tag: ["a"], q: "x", kind: "week" });
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
    expect(hasAnyFilter(parseReportSearch({ q: "   " }))).toBe(false);
    expect(hasAnyFilter(parseReportSearch({ client: [3] }))).toBe(true);
  });
});
