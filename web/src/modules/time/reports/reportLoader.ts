import { coreWithDefaults, settingsOptions } from "@/api/settings";
import { facetsOptions, type Facets, reportOptions, type ReportQuery } from "@/api/time";
import { warmed, type WarmableQuery } from "@/app/module-manifest";
import { browserTimeZone } from "../browserTimeZone";
import { startingFilters } from "./defaultFilters";
import { loadLastFilters } from "./lastFilters";
import { todayOf, type WeekStart } from "@/lib/period/period";
import { buildReportQuery } from "./reportQuery";
import { rangeOf } from "@/lib/period/periodSearch";
import { parseReportSearch, type ReportSearch } from "./reportSearch";

interface LoaderInput {
  search: ReportSearch;
  /** Undefined when the filter choices could not load. */
  facets: Facets | undefined;
  weekStart: WeekStart;
  now: Date;
  tz: string;
}

/**
 * The Report query the page will ask for, worked out the way the page does: remembered filters
 * apply when the URL names none, and the Period follows the week start. Pure, so the loader and
 * the page agree on one cache key.
 */
export function reportQueryForLoader({ search, facets, weekStart, now, tz }: LoaderInput): ReportQuery {
  const starting = facets ? startingFilters(search, loadLastFilters(), facets) : null;
  const effective = starting ? { ...search, ...starting } : search;
  return buildReportQuery(effective, rangeOf(effective, todayOf(now), weekStart), tz);
}

/**
 * Warms the Report page's main query from the URL's search. It first needs the settings (week
 * start) and the filter choices, which the page waits for too; if either fails the page falls
 * back to defaults, and so does this.
 */
export function warmReport(rawSearch: Record<string, unknown>): WarmableQuery {
  return async (queryClient) => {
    const [settings, facets] = await Promise.allSettled([warmed(queryClient, settingsOptions), warmed(queryClient, facetsOptions)]);
    const query = reportQueryForLoader({
      search: parseReportSearch(rawSearch),
      facets: facets.status === "fulfilled" ? facets.value : undefined,
      weekStart: coreWithDefaults(settings.status === "fulfilled" ? settings.value.core : undefined).week_start,
      now: new Date(),
      tz: browserTimeZone(),
    });
    await warmed(queryClient, reportOptions(query));
  };
}
