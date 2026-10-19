import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { callApi } from "../callApi";
import { api } from "../client";
import { overviewKey } from "./keys";
import type { OverviewQuery } from "./types";

/** The Overview of a range; the previous one stays on screen while the next loads. A null query waits. */
export function useOverview(query: OverviewQuery | null) {
  const asked = query ?? { from: "", to: "" };
  return useQuery({
    queryKey: overviewKey(asked),
    queryFn: () => callApi(() => api.GET("/api/finance/overview", { params: { query: asked } })),
    enabled: query !== null,
    placeholderData: keepPreviousData,
    retry: false,
  });
}
