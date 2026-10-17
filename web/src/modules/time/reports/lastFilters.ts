import { parseReportSearch, type ReportFilters } from "./reportSearch";

const STORAGE_KEY = "vitarchy.time.reports.filters";

/**
 * The filters the person last chose, or null if they never chose any. Storage
 * can be blocked or damaged, so every access is guarded. Description search is
 * deliberately not remembered: it is a momentary lookup.
 */
export function loadLastFilters(): ReportFilters | null {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === null) return null;
    const { project, tag, client } = parseReportSearch(JSON.parse(stored) as Record<string, unknown>);
    return { project, tag, client };
  } catch {
    return null;
  }
}

/** Remembers the filters for the next visit; silently does nothing if storage is unavailable. */
export function saveLastFilters({ project, tag, client }: ReportFilters): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ project, tag, client }));
  } catch {
    // Remembering is a convenience; the page works without it.
  }
}
