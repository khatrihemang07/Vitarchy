import type { Facets } from "@/api/time";
import { hasFilters, type ReportFilters, type ReportSearch } from "./reportSearch";

const DEFAULT_PROJECT_NAME = "office";

function defaultFilters(facets: Facets): ReportFilters | null {
  // Facets are newest first, so the fallback is the most recently used project.
  const chosen =
    facets.projects.find((p) => p.name.trim().toLowerCase() === DEFAULT_PROJECT_NAME) ?? facets.projects[0];
  return chosen ? { project: [chosen.id], tag: [], client: [] } : null;
}

function stillExisting(filters: ReportFilters, facets: Facets): ReportFilters {
  return {
    project: filters.project.filter((id) => facets.projects.some((p) => p.id === id)),
    tag: filters.tag.filter((name) => facets.tags.some((t) => t.name === name)),
    client: filters.client.filter((id) => facets.clients.some((c) => c.id === id)),
  };
}

/**
 * The filters to apply when the page opens without any in the URL, or null to
 * leave things as they are. The URL always wins; then the last filters; on a
 * first visit, the Office project (or else the most recently used one).
 */
export function startingFilters(
  search: ReportSearch,
  remembered: ReportFilters | null,
  facets: Facets,
): ReportFilters | null {
  if (hasFilters(search)) return null;
  if (!remembered) return defaultFilters(facets);
  if (!hasFilters(remembered)) return null;
  const usable = stillExisting(remembered, facets);
  return hasFilters(usable) ? usable : defaultFilters(facets);
}
