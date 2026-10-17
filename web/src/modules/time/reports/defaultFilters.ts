import type { Facets } from "@/api/time";
import { hasFilters, type ReportFilters, type ReportSearch } from "./reportSearch";

function stillExisting(filters: ReportFilters, facets: Facets): ReportFilters {
  return {
    project: filters.project.filter((id) => facets.projects.some((p) => p.id === id)),
    tag: filters.tag.filter((name) => facets.tags.some((t) => t.name === name)),
    client: filters.client.filter((id) => facets.clients.some((c) => c.id === id)),
  };
}

/**
 * The filters to apply when the page opens without any in the URL, or null to
 * leave things as they are. The URL always wins; then the last filters the
 * person chose, minus anything that no longer exists. Nothing is chosen for
 * them: a first visit is unfiltered.
 */
export function startingFilters(
  search: ReportSearch,
  remembered: ReportFilters | null,
  facets: Facets,
): ReportFilters | null {
  if (hasFilters(search) || !remembered || !hasFilters(remembered)) return null;
  const usable = stillExisting(remembered, facets);
  return hasFilters(usable) ? usable : null;
}
