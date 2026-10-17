import { useEffect, useMemo } from "react";
import type { Facets } from "@/api/time";
import { createLogger } from "@/lib/log";
import { startingFilters } from "./defaultFilters";
import { loadLastFilters } from "./lastFilters";
import type { ReportSearch } from "./reportSearch";

const log = createLogger("time");

/**
 * When the URL names no filters, the remembered ones (or the first-visit
 * default) apply. They are used at once and written to the URL once, so the
 * first Report is requested with them rather than without.
 */
export function useStartingFilters(
  search: ReportSearch,
  facets: Facets | undefined,
  update: (next: ReportSearch, options: { replace: boolean }) => void,
): ReportSearch {
  const starting = useMemo(() => (facets ? startingFilters(search, loadLastFilters(), facets) : null), [search, facets]);
  useEffect(() => {
    if (!starting) return;
    log.debug("starting filters applied", { projects: starting.project.length, tags: starting.tag.length, clients: starting.client.length });
    update({ ...search, ...starting }, { replace: true });
  }, [starting, search, update]);
  return starting ? { ...search, ...starting } : search;
}
