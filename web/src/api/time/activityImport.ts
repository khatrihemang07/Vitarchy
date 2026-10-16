import { useMutation, useQueryClient } from "@tanstack/react-query";

import { createLogger } from "@/lib/log";

import { callApi } from "../callApi";
import { api } from "../client";
import { toTimeFailure } from "./failure";
import { timeStatusKey } from "./keys";

const log = createLogger("time");

/** Imports Activity from the latest Snapshot; the status is refreshed whether it worked or failed. */
export function useImportActivity() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: () => callApi(() => api.POST("/api/time/activity/import")),
    onSuccess: (counts) => {
      log.info("activity import finished", { ...counts });
    },
    onError: (error) => {
      log.warn("activity import failed", { kind: toTimeFailure(error).kind });
    },
    onSettled: () => client.invalidateQueries({ queryKey: timeStatusKey }),
  });
}
