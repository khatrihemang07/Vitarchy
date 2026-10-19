import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createLogger } from "@/lib/log";
import { callApi } from "../callApi";
import { api } from "../client";
import { toFinanceFailure } from "./failure";
import { transactionsKeyPrefix, transferCandidatesKey } from "./keys";

const log = createLogger("finance");

/** The Transactions one Transaction could be linked to as a Transfer; loads only when `enabled` (its dialog is open). */
export function useTransferCandidates(transactionId: number, enabled: boolean) {
  return useQuery({
    queryKey: transferCandidatesKey(transactionId),
    queryFn: () =>
      callApi(() => api.GET("/api/finance/transactions/{id}/transfer-candidates", { params: { path: { id: transactionId } } })),
    enabled,
    retry: false,
  });
}

/** Links a Money-out Transaction to Money-in Transactions as a Transfer. */
export function useLinkTransfer() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (input: { debit_id: number; credit_ids: number[] }) =>
      callApi(() => api.POST("/api/finance/transfers", { body: input })),
    onSuccess: (linked) => {
      log.info("transfer linked", { transferId: linked.transfer.id, credits: linked.credit_ids.length });
    },
    onError: (error) => {
      log.warn("linking a transfer failed", { kind: toFinanceFailure(error).kind });
    },
    onSettled: () => client.invalidateQueries({ queryKey: transactionsKeyPrefix }),
  });
}

/** Unlinks a Transfer; detection never links the same pairs again. */
export function useUnlinkTransfer() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => callApi(() => api.DELETE("/api/finance/transfers/{id}", { params: { path: { id } } })),
    onSuccess: (_, id) => {
      log.info("transfer unlinked", { transferId: id });
    },
    onError: (error, id) => {
      log.warn("unlinking a transfer failed", { transferId: id, kind: toFinanceFailure(error).kind });
    },
    onSettled: () => client.invalidateQueries({ queryKey: transactionsKeyPrefix }),
  });
}
