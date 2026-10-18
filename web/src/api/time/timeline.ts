import { useMutation, useQueryClient } from "@tanstack/react-query";

import { createLogger } from "@/lib/log";

import { callApi } from "../callApi";
import { api } from "../client";
import { toTimeFailure } from "./failure";
import { timeStatusKey } from "./keys";

const log = createLogger("time");

/** Rebuilds the whole Timeline from Activity; the status is refreshed whether it worked or failed. */
export function useRebuildTimeline() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: () => callApi(() => api.POST("/api/time/timeline/rebuild")),
    onSuccess: (result) => {
      log.info("timeline rebuilt", { ...result });
    },
    onError: (error) => {
      log.warn("timeline rebuild failed", { kind: toTimeFailure(error).kind });
    },
    onSettled: () => client.invalidateQueries({ queryKey: timeStatusKey }),
  });
}
