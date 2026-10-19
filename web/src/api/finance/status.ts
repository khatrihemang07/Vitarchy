import { queryOptions, useQuery } from "@tanstack/react-query";
import { callApi } from "../callApi";
import { api } from "../client";
import { financeStatusKey } from "./keys";

/** How to load the Finance status (reader available, counts, latest runs); shared by the hook and by route loaders. */
export const financeStatusOptions = queryOptions({
  queryKey: financeStatusKey,
  queryFn: () => callApi(() => api.GET("/api/finance/status")),
  retry: false,
});

/** Query for the Finance status. */
export function useFinanceStatus() {
  return useQuery(financeStatusOptions);
}
