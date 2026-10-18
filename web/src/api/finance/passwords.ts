import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createLogger } from "@/lib/log";
import { callApi } from "../callApi";
import { api } from "../client";
import { toFinanceFailure } from "./failure";
import { passwordsKey } from "./keys";

const log = createLogger("finance");

/** The saved Statement passwords (labels only). */
export function usePasswords() {
  return useQuery({
    queryKey: passwordsKey,
    queryFn: () => callApi(() => api.GET("/api/finance/statement-passwords")),
    retry: false,
  });
}

/** Saves a Statement password. The secret is sent once and never logged. */
export function useAddPassword() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (input: { label: string; secret: string }) =>
      callApi(() => api.POST("/api/finance/statement-passwords", { body: input })),
    onSuccess: (saved) => {
      log.info("statement password saved", { passwordId: saved.id });
    },
    onError: (error) => {
      log.warn("saving a statement password failed", { kind: toFinanceFailure(error).kind });
    },
    onSettled: () => client.invalidateQueries({ queryKey: passwordsKey }),
  });
}

/** Forgets a saved Statement password. */
export function useDeletePassword() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (id: number) =>
      callApi(() => api.DELETE("/api/finance/statement-passwords/{id}", { params: { path: { id } } })),
    onSuccess: (_, id) => {
      log.info("statement password removed", { passwordId: id });
    },
    onError: (error, id) => {
      log.warn("removing a statement password failed", { passwordId: id, kind: toFinanceFailure(error).kind });
    },
    onSettled: () => client.invalidateQueries({ queryKey: passwordsKey }),
  });
}
