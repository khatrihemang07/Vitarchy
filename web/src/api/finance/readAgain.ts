import { useMutation, useQueryClient } from "@tanstack/react-query";
import { createLogger } from "@/lib/log";
import { callApi } from "../callApi";
import { api } from "../client";
import { toFinanceFailure } from "./failure";
import { accountsKey, financeStatusKey, statementsKeyPrefix, transactionsKeyPrefix } from "./keys";

const log = createLogger("finance");

/**
 * Reads a stored Statement again. For an unconfirmed one the server re-reads it (this is how a
 * locked file opens after a password is saved); for a confirmed one it returns what would change.
 */
export function useReadAgain() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (id: number) =>
      callApi(() => api.POST("/api/finance/statements/{id}/read-again", { params: { path: { id } } })),
    onSuccess: (_, id) => {
      log.info("read again previewed", { statementId: id });
    },
    onError: (error, id) => {
      log.warn("read again failed", { statementId: id, kind: toFinanceFailure(error).kind });
    },
    onSettled: () =>
      Promise.all([
        client.invalidateQueries({ queryKey: statementsKeyPrefix }),
        client.invalidateQueries({ queryKey: financeStatusKey }),
      ]),
  });
}

/** Applies a Read again to a confirmed Statement. */
export function useConfirmReadAgain() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (id: number) =>
      callApi(() => api.POST("/api/finance/statements/{id}/read-again/confirm", { params: { path: { id } } })),
    onSuccess: (counts, id) => {
      log.info("read again applied", { statementId: id, ...counts });
    },
    onError: (error, id) => {
      log.warn("read again apply failed", { statementId: id, kind: toFinanceFailure(error).kind });
    },
    onSettled: () =>
      Promise.all([
        client.invalidateQueries({ queryKey: accountsKey }),
        client.invalidateQueries({ queryKey: transactionsKeyPrefix }),
        client.invalidateQueries({ queryKey: financeStatusKey }),
      ]),
  });
}

/** Reads every Statement with a stored original again; returns one preview each and writes nothing to confirmed data. */
export function useReadAllAgain() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: () => callApi(() => api.POST("/api/finance/statements/read-again")),
    onSuccess: (items) => {
      log.info("read all again previewed", { statements: items.length });
    },
    onError: (error) => {
      log.warn("read all again failed", { kind: toFinanceFailure(error).kind });
    },
    onSettled: () =>
      Promise.all([
        client.invalidateQueries({ queryKey: statementsKeyPrefix }),
        client.invalidateQueries({ queryKey: financeStatusKey }),
      ]),
  });
}

/** Applies the Read all again of every confirmed Statement that has changes. */
export function useConfirmReadAllAgain() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: () => callApi(() => api.POST("/api/finance/statements/read-again/confirm")),
    onSuccess: (counts) => {
      log.info("read all again applied", { ...counts });
    },
    onError: (error) => {
      log.warn("read all again apply failed", { kind: toFinanceFailure(error).kind });
    },
    onSettled: () =>
      Promise.all([
        client.invalidateQueries({ queryKey: accountsKey }),
        client.invalidateQueries({ queryKey: statementsKeyPrefix }),
        client.invalidateQueries({ queryKey: transactionsKeyPrefix }),
        client.invalidateQueries({ queryKey: financeStatusKey }),
      ]),
  });
}
