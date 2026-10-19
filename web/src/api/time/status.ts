import { queryOptions, useQuery } from "@tanstack/react-query";
import { callApi } from "../callApi";
import { api } from "../client";
import { timeStatusKey } from "./keys";
import type { TimeStatus } from "./types";

/** Loads the Time status (connection, latest runs, how much is mirrored). */
export function fetchTimeStatus(): Promise<TimeStatus> {
  return callApi(() => api.GET("/api/time/status"));
}

/** How to load the Time status; shared by the hook and by route loaders. */
export const timeStatusOptions = queryOptions({ queryKey: timeStatusKey, queryFn: fetchTimeStatus, retry: false });

/** Query for the Time status; failures surface as `isError`, never a throw. */
export function useTimeStatus() {
  return useQuery(timeStatusOptions);
}
