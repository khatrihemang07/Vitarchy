import { useQuery } from "@tanstack/react-query";
import { callApi } from "../callApi";
import { api } from "../client";
import { timeStatusKey } from "./keys";
import type { TimeStatus } from "./types";

/** Loads the Time status (connection, latest runs, how much is mirrored). */
function fetchTimeStatus(): Promise<TimeStatus> {
  return callApi(() => api.GET("/api/time/status"));
}

/** Query for the Time status; failures surface as `isError`, never a throw. */
export function useTimeStatus() {
  return useQuery({ queryKey: timeStatusKey, queryFn: fetchTimeStatus, retry: false });
}
