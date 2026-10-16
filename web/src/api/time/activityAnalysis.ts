import { useQuery } from "@tanstack/react-query";
import { callApi } from "../callApi";
import { api } from "../client";
import { activityKey } from "./keys";
import type { ActivityOverview, WeekStart } from "./types";

/** A Period to analyse: local dates `YYYY-MM-DD` and the IANA zone the Days are measured in. */
export interface ActivityPeriod {
  from: string;
  to: string;
  tz: string;
}

/**
 * Totals, heatmap, Day bounds and weekly trend for a Period, from one request. Every panel that
 * calls this with the same Period shares that request; `select` picks the part a panel draws.
 */
export function useActivityOverview<T>(query: ActivityPeriod, weekStart: WeekStart, select: (overview: ActivityOverview) => T) {
  return useQuery({
    queryKey: activityKey("overview", { ...query, weekStart }),
    queryFn: () =>
      callApi(() => api.GET("/api/time/analysis/activity/overview", { params: { query: { ...query, week_start: weekStart } } })),
    select,
    retry: false,
  });
}

/** One Day's Activity as merged spans; waits until a Day is chosen. */
export function useActivityTimeline(day: string | undefined, tz: string) {
  return useQuery({
    queryKey: activityKey("timeline", { day, tz }),
    queryFn: () => callApi(() => api.GET("/api/time/analysis/activity/timeline", { params: { query: { day: day ?? "", tz } } })),
    enabled: day !== undefined,
    retry: false,
  });
}
