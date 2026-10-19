import { RequestError } from "@/api/callApi";
import { createLogger } from "@/lib/log";
import { QueryClient, type QueryExecuteOptions, type QueryKey } from "@tanstack/react-query";

const log = createLogger("router");

/** What the router hands every route loader. */
export interface RouterContext {
  queryClient: QueryClient;
}

/** Loads a query into the cache and gives its data; whatever is cached is used however old, and the page refreshes it. */
export function warmed<TData, TError, TQueryData, TQueryKey extends QueryKey>(
  queryClient: QueryClient,
  options: QueryExecuteOptions<TData, TError, TData, TQueryData, TQueryKey>,
): Promise<TData> {
  return queryClient.query({ ...options, staleTime: "static" });
}

/** A query a page needs, ready to start on a query client; made by `warm`. */
export type WarmableQuery = (queryClient: QueryClient) => Promise<unknown>;

/** Wraps a query's options (`queryOptions(...)`) so a loader can start it. */
export function warm<TData, TError, TQueryData, TQueryKey extends QueryKey>(
  options: QueryExecuteOptions<TData, TError, TData, TQueryData, TQueryKey>,
): WarmableQuery {
  return (queryClient) => warmed(queryClient, options);
}

function isRouterContext(context: unknown): context is RouterContext {
  return typeof context === "object" && context !== null && "queryClient" in context && context.queryClient instanceof QueryClient;
}

/**
 * Starts the queries and waits for them. A failure is logged (status only, never what was asked
 * or answered) and left to the page, which shows it with Try again.
 */
async function start(queryClient: QueryClient, queries: readonly WarmableQuery[]): Promise<void> {
  const results = await Promise.allSettled(queries.map((query) => query(queryClient)));
  for (const result of results) {
    if (result.status === "rejected") {
      const error: unknown = result.reason;
      log.warn("warm failed", { scope: "route-loader", ...(error instanceof RequestError && { kind: error.kind, status: error.status }) });
    }
  }
}

/**
 * A route loader that fills the query cache with a page's main queries, so the page opens with
 * data (or already loading) and hovering a tab warms it. It never rejects, so an error never
 * blocks navigation.
 */
export function warmCache(queries: readonly WarmableQuery[]) {
  return async ({ context }: { context: unknown }): Promise<void> => {
    if (isRouterContext(context)) await start(context.queryClient, queries);
  };
}

/** Like `warmCache`, for a page whose queries depend on what its `loaderDeps` returned (the URL's search). */
export function warmCacheWith<Deps>(queries: (deps: Deps) => readonly WarmableQuery[]) {
  return async ({ context, deps }: { context: unknown; deps: Deps }): Promise<void> => {
    if (isRouterContext(context)) await start(context.queryClient, queries(deps));
  };
}
