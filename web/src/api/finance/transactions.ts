import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { callApi } from "../callApi";
import { api } from "../client";
import { transactionsKey } from "./keys";
import type { TransactionQuery } from "./types";

/** One page of Transactions; the previous page stays on screen while the next loads. */
export function useTransactions(query: TransactionQuery) {
  return useQuery({
    queryKey: transactionsKey(query),
    queryFn: () => callApi(() => api.GET("/api/finance/transactions", { params: { query } })),
    placeholderData: keepPreviousData,
    retry: false,
  });
}
