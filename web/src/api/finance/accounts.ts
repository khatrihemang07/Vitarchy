import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createLogger } from "@/lib/log";
import { callApi } from "../callApi";
import { api } from "../client";
import { toFinanceFailure } from "./failure";
import { accountsKey } from "./keys";

const log = createLogger("finance");

/** Every Account with its figures. */
export function useAccounts() {
  return useQuery({
    queryKey: accountsKey,
    queryFn: () => callApi(() => api.GET("/api/finance/accounts")),
    retry: false,
  });
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
