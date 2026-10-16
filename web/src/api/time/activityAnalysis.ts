import { useQuery } from "@tanstack/react-query";
import { callApi } from "../callApi";
import { api } from "../client";
import { activityKey } from "./keys";

/** A Period to analyse: local dates `YYYY-MM-DD` and the IANA zone the Days are measured in. */
export interface ActivityPeriod {
  from: string;
  to: string;
  tz: string;
}

/** Screen time by app, then Context, then top titles. */
export function useActivityTotals(query: ActivityPeriod) {
  return useQuery({
    queryKey: activityKey("totals", query),
    queryFn: () => callApi(() => api.GET("/api/time/analysis/activity/totals", { params: { query } })),
    retry: false,
  });
}

/** Seconds of Activity by weekday (Monday first) and local hour. */
export function useActivityHeatmap(query: ActivityPeriod) {
  return useQuery({
    queryKey: activityKey("heatmap", query),
    queryFn: () => callApi(() => api.GET("/api/time/analysis/activity/heatmap", { params: { query } })),
    retry: false,
  });
}

/** First and last Activity of each Day that has any. */
export function useActivityBounds(query: ActivityPeriod) {
  return useQuery({
    queryKey: activityKey("bounds", query),
    queryFn: () => callApi(() => api.GET("/api/time/analysis/activity/bounds", { params: { query } })),
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

/** Weekly totals for the busiest apps; weeks start on `weekStart`. */
export function useActivityTrend(query: ActivityPeriod, weekStart: "monday" | "sunday") {
  return useQuery({
    queryKey: activityKey("trend", { ...query, weekStart }),
    queryFn: () =>
      callApi(() => api.GET("/api/time/analysis/activity/trend", { params: { query: { ...query, week_start: weekStart } } })),
    retry: false,
  });
}
