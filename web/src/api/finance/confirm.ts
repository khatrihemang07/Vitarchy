import { useMutation, useQueryClient } from "@tanstack/react-query";
import { createLogger } from "@/lib/log";
import { callApi } from "../callApi";
import { api } from "../client";
import { toFinanceFailure } from "./failure";
import { accountsKey, financeStatusKey, statementsKeyPrefix, transactionsKeyPrefix } from "./keys";
import type { SectionChoice } from "./types";

const log = createLogger("finance");

interface ConfirmInput {
  id: number;
  sections: SectionChoice[];
}

/** Confirms a Statement with one choice per Account section. */
export function useConfirmStatement() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: ({ id, sections }: ConfirmInput) =>
      callApi(() => api.POST("/api/finance/statements/{id}/confirm", { params: { path: { id } }, body: { sections } })),
    onSuccess: (counts, { id }) => {
      log.info("statement confirmed", { statementId: id, ...counts });
    },
    onError: (error, { id }) => {
      log.warn("confirm failed", { statementId: id, kind: toFinanceFailure(error).kind });
    },
    onSettled: () =>
      Promise.all([
        client.invalidateQueries({ queryKey: statementsKeyPrefix }),
        client.invalidateQueries({ queryKey: financeStatusKey }),
        client.invalidateQueries({ queryKey: accountsKey }),
        client.invalidateQueries({ queryKey: transactionsKeyPrefix }),
      ]),
  });
}
