import { keepPreviousData, queryOptions, useQuery } from "@tanstack/react-query";
import { callApi } from "../callApi";
import { api } from "../client";
import { overviewKey } from "./keys";
import type { OverviewQuery } from "./types";

/** How to load the Overview of a range; shared by the hook and by route loaders that warm the cache. */
export function overviewOptions(query: OverviewQuery) {
  return queryOptions({
    queryKey: overviewKey(query),
    queryFn: () => callApi(() => api.GET("/api/finance/overview", { params: { query } })),
    retry: false,
  });
}

/** The Overview of a range; the previous one stays on screen while the next loads. A null query waits. */
export function useOverview(query: OverviewQuery | null) {
  return useQuery({
    ...overviewOptions(query ?? { from: "", to: "" }),
    enabled: query !== null,
    placeholderData: keepPreviousData,
  });
}
