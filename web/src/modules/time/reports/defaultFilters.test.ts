import type { Facets } from "@/api/time";
import { startingFilters } from "./defaultFilters";
import { parseReportSearch } from "./reportSearch";

const project = (id: number, name: string) => ({ id, name, active: true, last_used: "2026-10-01T00:00:00Z" });
const facets = (names: [number, string][]): Facets => ({ projects: names.map(([id, n]) => project(id, n)), tags: [], clients: [] });
const bare = parseReportSearch({});
const none = { project: [], tag: [], client: [] };

describe("startingFilters", () => {
  it("leaves a URL that already filters alone", () => {
    expect(startingFilters(parseReportSearch({ tag: ["a"] }), { ...none, project: [1] }, facets([[1, "Office"]]))).toBeNull();
  });

  it("applies the remembered filters", () => {
    expect(startingFilters(bare, { ...none, project: [2] }, facets([[1, "Office"], [2, "Gym"]]))).toEqual({ ...none, project: [2] });
  });

  it("does nothing when what was remembered is no filter", () => {
    expect(startingFilters(bare, none, facets([[1, "Office"]]))).toBeNull();
  });

  it("drops remembered choices that no longer exist", () => {
    expect(startingFilters(bare, { ...none, project: [1, 99] }, facets([[1, "Gym"]]))).toEqual({ ...none, project: [1] });
  });

  it("applies nothing when every remembered choice is gone", () => {
    expect(startingFilters(bare, { ...none, project: [99] }, facets([[1, "Gym"], [2, "Office"]]))).toBeNull();
  });

  it("opens unfiltered on a first visit, even when an Office project exists", () => {
    expect(startingFilters(bare, null, facets([[1, "Gym"], [2, "Office"]]))).toBeNull();
  });
});
