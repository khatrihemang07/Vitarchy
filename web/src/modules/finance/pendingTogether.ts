/**
 * Several queries as one for `QueryNotices`: pending until all have answered, never failed. A page that
 * shows several lists uses it to show one skeleton for all of them; each list still shows its own failure.
 */
export function pendingTogether(queries: readonly { isPending: boolean }[]) {
  return {
    isPending: queries.some((query) => query.isPending),
    isError: false,
    error: null,
    refetch: () => Promise.resolve(),
  };
}
