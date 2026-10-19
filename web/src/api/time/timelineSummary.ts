import { keepPreviousData, queryOptions, useQuery } from "@tanstack/react-query";
import { createLogger } from "@/lib/log";
import { callApi } from "../callApi";
import { api } from "../client";
import { timeTimelineKey } from "./keys";
import type { TimelineQuery, TimelineSummary } from "./types";

const log = createLogger("time");

/** Loads the analysed Timeline of one Period. Counts are logged, never apps, titles or addresses. */
export async function fetchTimelineSummary(query: TimelineQuery): Promise<TimelineSummary> {
  const summary = await callApi(() => api.GET("/api/time/timeline", { params: { query } }));
  log.debug("timeline summary loaded", { kind: query.kind, days: summary.days.length, apps: summary.apps.length, segments: summary.segments?.length ?? 0 });
  return summary;
}

/** How to load a Timeline summary (disabled for null); shared by the hook and by the route loader that warms it. */
export function timelineSummaryOptions(query: TimelineQuery | null) {
  return queryOptions({
    queryKey: timeTimelineKey(query),
    queryFn: () => fetchTimelineSummary(query as TimelineQuery),
    enabled: query !== null,
    retry: false,
  });
}

/**
 * Query for a Timeline summary. The previous summary stays on screen while the next
 * Period loads, so moving between Periods does not flash an empty page.
 */
export function useTimelineSummary(query: TimelineQuery | null) {
  return useQuery({ ...timelineSummaryOptions(query), placeholderData: keepPreviousData });
}
