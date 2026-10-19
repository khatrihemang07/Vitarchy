import { queryOptions, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createLogger } from "@/lib/log";
import { callApi } from "../callApi";
import { api } from "../client";
import { toFinanceFailure } from "./failure";
import { financeStatusKey, statementPreviewKey, statementsKey, statementsKeyPrefix } from "./keys";
import type { StatementPreview } from "./types";

const log = createLogger("finance");

/** How to load the Statements waiting to be confirmed; shared by the hook and by route loaders. */
export const unconfirmedStatementsOptions = queryOptions({
  queryKey: statementsKey("unconfirmed"),
  queryFn: () => callApi(() => api.GET("/api/finance/statements", { params: { query: { status: "unconfirmed" } } })),
  retry: false,
});

/** How to load the confirmed Statements; shared by the hook and by route loaders. */
export const confirmedStatementsOptions = queryOptions({
  queryKey: statementsKey("confirmed"),
  queryFn: () => callApi(() => api.GET("/api/finance/statements", { params: { query: { status: "confirmed" } } })),
  retry: false,
});

/** Statements waiting to be confirmed, newest upload first. */
export function useUnconfirmedStatements() {
  return useQuery(unconfirmedStatementsOptions);
}

/** Confirmed Statements, newest upload first. */
export function useConfirmedStatements() {
  return useQuery(confirmedStatementsOptions);
}

/**
 * One Statement as a preview; loads only when `enabled`. `initial` seeds it (an upload's
 * preview), and it is then refetched after every confirm so Account matches stay current.
 */
export function useStatementPreview(id: number, enabled: boolean, initial?: StatementPreview) {
  return useQuery({
    queryKey: statementPreviewKey(id),
    initialData: initial,
    // Always refetched when shown: a seeded upload preview must pick up Account matches made since it was read.
    staleTime: 0,
    queryFn: () => callApi(() => api.GET("/api/finance/statements/{id}", { params: { path: { id } } })),
    enabled,
    retry: false,
  });
}

/** Discards an unconfirmed Statement and its stored file. */
export function useDiscardStatement() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => callApi(() => api.DELETE("/api/finance/statements/{id}", { params: { path: { id } } })),
    onSuccess: (_, id) => {
      log.info("statement discarded", { statementId: id });
    },
    onError: (error, id) => {
      log.warn("discard failed", { statementId: id, kind: toFinanceFailure(error).kind });
    },
    onSettled: () =>
      Promise.all([
        client.invalidateQueries({ queryKey: statementsKeyPrefix }),
        client.invalidateQueries({ queryKey: financeStatusKey }),
      ]),
  });
}
