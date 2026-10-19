import { queryOptions, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createLogger } from "@/lib/log";
import { callApi } from "../callApi";
import { api } from "../client";
import { toFinanceFailure } from "./failure";
import { accountsKey, transactionsKeyPrefix } from "./keys";

const log = createLogger("finance");

/** How to load every Account with its figures; shared by the hook and by route loaders. */
export const accountsOptions = queryOptions({
  queryKey: accountsKey,
  queryFn: () => callApi(() => api.GET("/api/finance/accounts")),
  retry: false,
});

/** Every Account with its figures. */
export function useAccounts() {
  return useQuery(accountsOptions);
}

/** Renames an Account. */
export function useRenameAccount() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: ({ id, name }: { id: number; name: string }) =>
      callApi(() => api.PATCH("/api/finance/accounts/{id}", { params: { path: { id } }, body: { name } })),
    onSuccess: (_, { id }) => {
      log.info("account renamed", { accountId: id });
    },
    onError: (error, { id }) => {
      log.warn("rename failed", { accountId: id, kind: toFinanceFailure(error).kind });
    },
    onSettled: () => client.invalidateQueries({ queryKey: accountsKey }),
  });
}

/** Hides an Account or shows it again; hiding changes the Transactions page too. */
export function useSetAccountHidden() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: ({ id, hidden }: { id: number; hidden: boolean }) =>
      callApi(() => api.PATCH("/api/finance/accounts/{id}", { params: { path: { id } }, body: { hidden } })),
    onSuccess: (_, { id, hidden }) => {
      log.info("account visibility changed", { accountId: id, hidden });
    },
    onError: (error, { id }) => {
      log.warn("hiding failed", { accountId: id, kind: toFinanceFailure(error).kind });
    },
    onSettled: () =>
      Promise.all([
        client.invalidateQueries({ queryKey: accountsKey }),
        client.invalidateQueries({ queryKey: transactionsKeyPrefix }),
      ]),
  });
}
