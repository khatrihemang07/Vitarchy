import type { OverviewQuery, TransactionQuery } from "@/api/finance";
import type { DayRange } from "@/lib/period/period";
import type { TransactionsSearch } from "./transactionsSearch";

/** Rows on one page of the Transactions table. */
export const PAGE_SIZE = 50;

/** What narrows the Transactions and the Overview alike: the Accounts and the search. */
function narrowing(search: TransactionsSearch) {
  const q = search.q.trim();
  return { ...(search.accounts.length > 0 && { account_id: search.accounts }), ...(q && { q }) };
}

/** The API query for one page of Transactions over `range`. */
export function toTransactionQuery(search: TransactionsSearch, range: DayRange): TransactionQuery {
  return {
    ...range,
    ...narrowing(search),
    ...(search.direction !== "all" && { direction: search.direction }),
    ...(search.sort !== "date_desc" && { sort: search.sort }),
    limit: PAGE_SIZE,
    offset: ((search.page ?? 1) - 1) * PAGE_SIZE,
  };
}

/** The API query for the Overview over `range`: the same Accounts and search, but every direction. */
export function toOverviewQuery(search: TransactionsSearch, range: DayRange): OverviewQuery {
  return { ...range, ...narrowing(search) };
}

/** Which rows a page shows, e.g. "51 to 100 of 230", and whether there are more pages either way. */
export function pageRange(total: number, page: number) {
  const first = total === 0 ? 0 : (page - 1) * PAGE_SIZE + 1;
  const last = Math.min(total, page * PAGE_SIZE);
  return { first, last, hasPrevious: page > 1, hasNext: last < total, pageCount: Math.max(1, Math.ceil(total / PAGE_SIZE)) };
}
