import type { ReportQuery } from "@/api/time";
import type { DayRange } from "./period";
import type { ReportSearch } from "./reportSearch";

/** The server query for a page state; lists and text are sent only when they have content. */
export function buildReportQuery(search: ReportSearch, range: DayRange, tz: string): ReportQuery {
  const q = search.q.trim();
  return {
    kind: search.kind,
    from: range.from,
    to: range.to,
    tz,
    ...(search.project.length > 0 && { project: search.project }),
    ...(search.tag.length > 0 && { tag: search.tag }),
    ...(search.client.length > 0 && { client: search.client }),
    ...(q && { q }),
    group1: search.group1,
    ...(search.group2 && { group2: search.group2 }),
  };
}
