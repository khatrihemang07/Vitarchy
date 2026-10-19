import { accountsOptions, overviewOptions, transactionsOptions, type OverviewQuery, type TransactionQuery } from "@/api/finance";
import { coreWithDefaults, settingsOptions } from "@/api/settings";
import { warmed, type WarmableQuery } from "@/app/module-manifest";
import type { WeekStart } from "@/lib/period/period";
import { rangeOf } from "@/lib/period/periodSearch";
import { todayOf } from "@/lib/period/period";
import { toOverviewQuery, toTransactionQuery } from "./transactionsQuery";
import { parseTransactionsSearch, type TransactionsSearch } from "./transactionsSearch";

/** The two queries the page asks for, worked out the way the page does, so the loader and the page share cache keys. */
export function transactionQueriesFor(search: TransactionsSearch, weekStart: WeekStart, now: Date): { transactions: TransactionQuery; overview: OverviewQuery } {
  const range = rangeOf(search, todayOf(now), weekStart);
  return { transactions: toTransactionQuery(search, range), overview: toOverviewQuery(search, range) };
}

/**
 * Warms the Transactions page from the URL's search: the Accounts, the page and the Overview.
 * A week Period needs the week start first, which the page waits for too; if Settings fail the
 * page falls back to the default, and so does this.
 */
export function warmTransactions(rawSearch: Record<string, unknown>): WarmableQuery {
  return async (queryClient) => {
    const [settings] = await Promise.allSettled([warmed(queryClient, settingsOptions)]);
    const weekStart = coreWithDefaults(settings.status === "fulfilled" ? settings.value.core : undefined).week_start;
    const queries = transactionQueriesFor(parseTransactionsSearch(rawSearch), weekStart, new Date());
    await Promise.all([
      warmed(queryClient, accountsOptions),
      warmed(queryClient, transactionsOptions(queries.transactions)),
      warmed(queryClient, overviewOptions(queries.overview)),
    ]);
  };
}
