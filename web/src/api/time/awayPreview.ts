import { useQuery } from "@tanstack/react-query";
import { callApi } from "../callApi";
import { api } from "../client";
import { timeAwayPreviewKey } from "./keys";
import type { AwayPreview } from "./types";

/** Loads the Away rules and what each threshold would give over the preview window. */
export function fetchAwayPreview(): Promise<AwayPreview> {
  return callApi(() => api.GET("/api/time/timeline/away-preview"));
}

/** Query for the Away preview; failures surface as `isError`, never a throw. */
export function useAwayPreview() {
  return useQuery({ queryKey: timeAwayPreviewKey, queryFn: fetchAwayPreview, retry: false });
}
