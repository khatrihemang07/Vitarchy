import { keepPreviousData, queryOptions, useQuery } from "@tanstack/react-query";
import { callApi } from "../callApi";
import { api } from "../client";
import { transactionsKey } from "./keys";
import type { TransactionQuery } from "./types";

/** How to load one page of Transactions; shared by the hook and by route loaders that warm the cache. */
export function transactionsOptions(query: TransactionQuery) {
  return queryOptions({
    queryKey: transactionsKey(query),
    queryFn: () => callApi(() => api.GET("/api/finance/transactions", { params: { query } })),
    retry: false,
  });
}

/** One page of Transactions; the previous page stays on screen while the next loads. */
export function useTransactions(query: TransactionQuery) {
  return useQuery({ ...transactionsOptions(query), placeholderData: keepPreviousData });
}
