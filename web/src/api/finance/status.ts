import { useQuery } from "@tanstack/react-query";
import { callApi } from "../callApi";
import { api } from "../client";
import { financeStatusKey } from "./keys";

/** Query for the Finance status (reader available, counts, latest runs). */
export function useFinanceStatus() {
  return useQuery({
    queryKey: financeStatusKey,
    queryFn: () => callApi(() => api.GET("/api/finance/status")),
    retry: false,
  });
}
