import { useMutation, useQueryClient } from "@tanstack/react-query";

import { createLogger } from "@/lib/log";

import { callApi } from "../callApi";
import { api } from "../client";
import { toTimeFailure } from "./failure";
import { timeStatusKey } from "./keys";
import type { Source } from "./types";

const log = createLogger("time");

/** Imports a Source's Activity from its latest Snapshot; the status is refreshed whether it worked or failed. */
export function useImportActivity(source: Source) {
  const client = useQueryClient();
  return useMutation({
    mutationFn: () => callApi(() => api.POST("/api/time/activity/{source}/import", { params: { path: { source } } })),
    onSuccess: (counts) => {
      log.info("activity import finished", { source, ...counts });
    },
    onError: (error) => {
      log.warn("activity import failed", { source, kind: toTimeFailure(error).kind });
    },
    onSettled: () => client.invalidateQueries({ queryKey: timeStatusKey }),
  });
}
