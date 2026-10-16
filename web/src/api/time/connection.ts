import { useMutation, useQueryClient } from "@tanstack/react-query";
import { createLogger } from "@/lib/log";
import { callApi } from "../callApi";
import { api } from "../client";
import { timeStatusKey } from "./keys";

const log = createLogger("time");

/** Checks a Toggl token and stores it server-side. The token is never kept in the cache. */
export function useSetConnection() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (apiToken: string) =>
      callApi(() => api.PUT("/api/time/connection", { body: { api_token: apiToken } })),
    onSuccess: () => {
      log.info("toggl connected");
      return client.invalidateQueries({ queryKey: timeStatusKey });
    },
    onError: (error) => {
      log.warn("connect failed", { kind: error.name });
    },
  });
}

/** Forgets the token; synced Time Entries stay. */
export function useDisconnect() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: () => callApi(() => api.DELETE("/api/time/connection")),
    onSuccess: () => {
      log.info("toggl disconnected");
      return client.invalidateQueries({ queryKey: timeStatusKey });
    },
  });
}
