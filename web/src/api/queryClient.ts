import { QueryClient } from "@tanstack/react-query";

/**
 * How long a result counts as fresh. A page that mounts within this time of a route loader or a
 * sibling section fetching the same thing uses it instead of asking the server again; anything
 * the person changes invalidates its queries explicitly, which refetches whatever the age.
 */
export const FRESH_MS = 5_000;

/** The app's query client. */
export function createQueryClient(): QueryClient {
  return new QueryClient({ defaultOptions: { queries: { staleTime: FRESH_MS } } });
}
