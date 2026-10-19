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

/** One page of Transactions; the previous page stays on screen while the next loads. A null query waits (e.g. for the week start). */
export function useTransactions(query: TransactionQuery | null) {
  return useQuery({ ...transactionsOptions(query ?? {}), enabled: query !== null, placeholderData: keepPreviousData });
}
