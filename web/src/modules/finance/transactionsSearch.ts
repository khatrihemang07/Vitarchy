/** Rows on one page of the Transactions table. */
export const PAGE_SIZE = 50;

/** What the URL of the Transactions page says. */
export interface TransactionsSearch {
  /** Only this Account. */
  account?: number;
  /** First day, `YYYY-MM-DD`. */
  from?: string;
  /** Last day, `YYYY-MM-DD`. */
  to?: string;
  /** 1-based page; absent for the first. */
  page?: number;
}

function validDay(value: unknown): string | undefined {
  if (typeof value !== "string" || !/^\d{4}-\d{2}-\d{2}$/.test(value)) return undefined;
  const date = new Date(`${value}T00:00:00Z`);
  return Number.isNaN(date.getTime()) || date.toISOString().slice(0, 10) !== value ? undefined : value;
}

function positiveInt(value: unknown): number | undefined {
  const n = typeof value === "number" || typeof value === "string" ? Number(value) : Number.NaN;
  return Number.isInteger(n) && n > 0 ? n : undefined;
}

/** Turns whatever the URL held into a valid state; the URL is user-editable, so nothing is trusted. */
export function parseTransactionsSearch(raw: Record<string, unknown>): TransactionsSearch {
  const account = positiveInt(raw.account);
  let from = validDay(raw.from);
  let to = validDay(raw.to);
  // The server refuses a backwards range, so a swapped one is put right.
  if (from && to && from > to) [from, to] = [to, from];
  const page = positiveInt(raw.page);
  return {
    ...(account && { account }),
    ...(from && { from }),
    ...(to && { to }),
    ...(page && page > 1 && { page }),
  };
}

/** The URL form of a state: defaults are left out. */
export function transactionsParams(search: TransactionsSearch): Record<string, unknown> {
  return { ...parseTransactionsSearch({ ...search }) };
}

/** The state after a filter changes: back to the first page. */
export function withFilter(search: TransactionsSearch, change: { [K in "account" | "from" | "to"]?: TransactionsSearch[K] | undefined }): TransactionsSearch {
  const next: Record<string, unknown> = { ...search, ...change };
  delete next.page;
  return parseTransactionsSearch(next);
}

/** The API query for a state. */
export function toTransactionQuery(search: TransactionsSearch) {
  return {
    ...(search.account && { account_id: search.account }),
    ...(search.from && { from: search.from }),
    ...(search.to && { to: search.to }),
    limit: PAGE_SIZE,
    offset: ((search.page ?? 1) - 1) * PAGE_SIZE,
  };
}

/** Which rows a page shows, e.g. "51 to 100 of 230", and whether there are more pages either way. */
export function pageRange(total: number, page: number) {
  const first = total === 0 ? 0 : (page - 1) * PAGE_SIZE + 1;
  const last = Math.min(total, page * PAGE_SIZE);
  return { first, last, hasPrevious: page > 1, hasNext: last < total, pageCount: Math.max(1, Math.ceil(total / PAGE_SIZE)) };
}
