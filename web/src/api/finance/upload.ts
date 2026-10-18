import { useMutation, useQueryClient } from "@tanstack/react-query";
import { createLogger } from "@/lib/log";
import { callApi } from "../callApi";
import { api } from "../client";
import { toFinanceFailure } from "./failure";
import { financeStatusKey, statementsKeyPrefix } from "./keys";

const log = createLogger("finance");

/** The multipart form the upload route reads: one `file` part per file. */
export function uploadForm(files: readonly File[]): FormData {
  const form = new FormData();
  for (const file of files) form.append("file", file, file.name);
  return form;
}

/**
 * Uploads Statement files in one multipart request and returns one preview per file.
 * The generated body type is a list of strings, so the serializer builds the form itself.
 */
export function postStatements(files: readonly File[]) {
  return callApi(() => api.POST("/api/finance/statements", { body: { file: [] }, bodySerializer: () => uploadForm(files) }));
}

/** Mutation that uploads Statement files and updates the lists they appear in. */
export function useUploadStatements() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: postStatements,
    onSuccess: (previews) => {
      log.info("statements uploaded", { files: previews.length });
    },
    onError: (error) => {
      log.warn("upload failed", { kind: toFinanceFailure(error).kind });
    },
    onSettled: () =>
      Promise.all([
        client.invalidateQueries({ queryKey: statementsKeyPrefix }),
        client.invalidateQueries({ queryKey: financeStatusKey }),
      ]),
  });
}
