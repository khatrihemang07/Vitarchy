import type { Grouping } from "@/api/time";
import { hasFilters, type ReportFilters, type ReportSearch } from "./reportSearch";

/** Which filter a list of chosen values belongs to. */
export type FacetKey = keyof ReportFilters;

/** The state with one filter replaced; projects and clients arrive as text and are stored as ids. */
export function withFacet(search: ReportSearch, key: FacetKey, values: string[]): ReportSearch {
  return key === "tag" ? { ...search, tag: values } : { ...search, [key]: values.map(Number) };
}

/** The state grouped by `group1` and optionally `group2`, which may never repeat the first level. */
export function withGroups(search: ReportSearch, group1: Grouping, group2: Grouping | undefined): ReportSearch {
  const next: ReportSearch = { ...search, group1 };
  delete next.group2;
  return group2 && group2 !== group1 ? { ...next, group2 } : next;
}

/** The state with no project, tag, client or search. */
export function clearFilters(search: ReportSearch): ReportSearch {
  return { ...search, project: [], tag: [], client: [], q: "" };
}

/** True when anything narrows the Report. */
export function hasAnyFilter(search: ReportSearch): boolean {
  return hasFilters(search) || search.q.trim() !== "";
}
