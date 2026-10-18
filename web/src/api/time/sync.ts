import { useMutation, useQueryClient } from "@tanstack/react-query";
import { createLogger } from "@/lib/log";
import { callApi } from "../callApi";
import { api } from "../client";
import { toTimeFailure } from "./failure";
import { timeStatusKey, timeTimelineKeyPrefix } from "./keys";

const log = createLogger("time");

/** Runs a Sync; the status is refreshed whether it worked or failed (a failed run is recorded too), and the Timeline too, since its Day lane shows Time Entries. */
export function useSyncTime() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: () => callApi(() => api.POST("/api/time/sync")),
    onSuccess: (counts) => {
      log.info("sync finished", { ...counts });
    },
    onError: (error) => {
      log.warn("sync failed", { kind: toTimeFailure(error).kind });
    },
    onSettled: () =>
      Promise.all([client.invalidateQueries({ queryKey: timeStatusKey }), client.invalidateQueries({ queryKey: timeTimelineKeyPrefix })]),
  });
}
