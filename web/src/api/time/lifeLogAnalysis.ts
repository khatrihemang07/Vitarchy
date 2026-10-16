import { useQuery } from "@tanstack/react-query";
import { callApi } from "../callApi";
import { api } from "../client";
import type { ActivityPeriod } from "./activityAnalysis";
import { lifeLogKey } from "./keys";
import type { LifeLogOverview } from "./types";

/**
 * The daily split, totals and description habits for a Period, from one request. Every panel that
 * calls this with the same Period shares that request; `select` picks the part a panel draws.
 */
export function useLifeLogOverview<T>(query: ActivityPeriod, select: (overview: LifeLogOverview) => T) {
  return useQuery({
    queryKey: lifeLogKey("overview", query),
    queryFn: () => callApi(() => api.GET("/api/time/analysis/lifelog/overview", { params: { query } })),
    select,
    retry: false,
  });
}

/** One Day's Time Entries; waits until a Day is chosen. */
export function useLifeLogTimeline(day: string | undefined, tz: string) {
  return useQuery({
    queryKey: lifeLogKey("timeline", { day, tz }),
    queryFn: () => callApi(() => api.GET("/api/time/analysis/lifelog/timeline", { params: { query: { day: day ?? "", tz } } })),
    enabled: day !== undefined,
    retry: false,
  });
}
