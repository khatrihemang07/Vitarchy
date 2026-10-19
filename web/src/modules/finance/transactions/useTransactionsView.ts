import { useAccounts, useOverview, useTransactions } from "@/api/finance";
import { containsDay, todayOf } from "@/lib/period/period";
import { rangeOf } from "@/lib/period/periodSearch";
import { useClock } from "@/lib/useClock";
import { toOverviewQuery, toTransactionQuery } from "./transactionsQuery";
import { useTransactionsHandlers } from "./useTransactionsHandlers";
import { useTransactionsSearch } from "./useTransactionsSearch";
import { useWeekStart } from "./useWeekStart";

/** Everything the Transactions page shows: the URL's state, the Period it covers, the controls' handlers and the three queries. */
export function useTransactionsView() {
  const { search, update } = useTransactionsSearch();
  const { weekStart, ready } = useWeekStart();
  const today = todayOf(useClock());
  const range = rangeOf(search, today, weekStart);
  const handlers = useTransactionsHandlers({ search, range, today, weekStart, update });
  // A week starts on a Setting, so wait for it rather than ask for one week and then another.
  const settled = search.kind !== "week" || ready;
  return {
    search,
    range,
    today,
    handlers,
    isCurrent: search.kind !== "custom" && containsDay(range, today),
    accounts: useAccounts(),
    page: useTransactions(settled ? toTransactionQuery(search, range) : null),
    overview: useOverview(settled ? toOverviewQuery(search, range) : null),
  };
}
