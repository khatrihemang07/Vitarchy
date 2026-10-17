import type { Facets } from "@/api/time";
import { facetDescriptors } from "./facetOptions";
import { parseReportSearch } from "./reportSearch";

const facets: Facets = {
  projects: [
    { id: 1, name: "Office", active: true, last_used: "2026-10-01T00:00:00Z" },
    { id: 2, name: "Old", active: false, last_used: "2026-09-01T00:00:00Z" },
  ],
  tags: [{ name: "deep", last_used: "2026-10-01T00:00:00Z" }],
  clients: [{ id: 5, name: "Acme", archived: true, last_used: "2026-10-01T00:00:00Z" }],
};

describe("facetDescriptors", () => {
  const descriptors = facetDescriptors(facets, parseReportSearch({ project: [2], tag: ["deep"] }));

  it("lists projects, tags and clients in that order", () => {
    expect(descriptors.map((d) => d.key)).toEqual(["project", "tag", "client"]);
  });

  it("offers values as text and marks archived choices", () => {
    expect(descriptors[0]?.options).toEqual([
      { value: "1", label: "Office" },
      { value: "2", label: "Old", hint: "archived" },
    ]);
    expect(descriptors[2]?.options).toEqual([{ value: "5", label: "Acme", hint: "archived" }]);
  });

  it("carries what is selected", () => {
    expect(descriptors.map((d) => d.selected)).toEqual([["2"], ["deep"], []]);
  });
});
