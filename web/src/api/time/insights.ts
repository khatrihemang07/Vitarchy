import { useQuery } from "@tanstack/react-query";
import { callApi } from "../callApi";
import { api } from "../client";
import type { ActivityPeriod } from "./activityAnalysis";
import { insightsKey } from "./keys";

/** The most notable Insights for a Period, most notable first. */
export function useInsights(period: ActivityPeriod) {
  return useQuery({
    queryKey: insightsKey(period),
    queryFn: () => callApi(() => api.GET("/api/time/analysis/insights", { params: { query: period } })),
    select: (body) => body.insights,
    retry: false,
  });
}
