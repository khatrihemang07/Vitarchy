import { queryOptions, useQuery } from "@tanstack/react-query";
import { callApi } from "../callApi";
import { api } from "../client";
import { timeFacetsKey } from "./keys";
import type { Facets } from "./types";

/** Loads the projects, tags and clients that have Time Entries, newest use first. */
export function fetchFacets(): Promise<Facets> {
  return callApi(() => api.GET("/api/time/facets"));
}

/** How to load the filter choices; shared by the hook and by route loaders. */
export const facetsOptions = queryOptions({ queryKey: timeFacetsKey, queryFn: fetchFacets, retry: false });

/** Query for the filter choices; failures surface as `isError`. */
export function useFacets() {
  return useQuery(facetsOptions);
}
