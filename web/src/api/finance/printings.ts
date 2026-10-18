import { useQuery } from "@tanstack/react-query";
import { createLogger } from "@/lib/log";
import { callApi, RequestError } from "../callApi";
import { api } from "../client";
import { printingsKey } from "./keys";

const log = createLogger("finance");

/** Every Printing of one Transaction, oldest first; loads only when `enabled` (the row is expanded). */
export function usePrintings(transactionId: number, enabled: boolean) {
  return useQuery({
    queryKey: printingsKey(transactionId),
    queryFn: async () => {
      try {
        return await callApi(() => api.GET("/api/finance/transactions/{id}/printings", { params: { path: { id: transactionId } } }));
      } catch (error) {
        log.warn("printings failed to load", { transactionId, status: error instanceof RequestError ? error.status : undefined });
        throw error;
      }
    },
    enabled,
    retry: false,
  });
}
