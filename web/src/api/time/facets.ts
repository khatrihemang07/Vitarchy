import { useQuery } from "@tanstack/react-query";
import { callApi } from "../callApi";
import { api } from "../client";
import { timeFacetsKey } from "./keys";
import type { Facets } from "./types";

/** Loads the projects, tags and clients that have Time Entries, newest use first. */
function fetchFacets(): Promise<Facets> {
  return callApi(() => api.GET("/api/time/facets"));
}

/** Query for the filter choices; failures surface as `isError`. */
export function useFacets() {
  return useQuery({ queryKey: timeFacetsKey, queryFn: fetchFacets, retry: false });
}
