import type { TransactionQuery } from "@/api/finance";
import { parsePeriodSearch, type PeriodSearch, periodParams } from "@/lib/period/periodSearch";

/** Which Transactions to keep by what they mean for the person's money. */
export type DirectionFilter = "all" | "in" | "out" | "transfer";

/** How the Transactions are ordered. */
export type TransactionSort = NonNullable<TransactionQuery["sort"]>;

/** What the URL of the Transactions page says. The Period is a month unless the URL says otherwise. */
export interface TransactionsSearch extends PeriodSearch {
  /** Only these Accounts; empty means all. */
  accounts: number[];
  /** Description search, ignoring case. */
  q: string;
  direction: DirectionFilter;
  sort: TransactionSort;
  /** 1-based page; absent for the first. */
  page?: number;
}

const DIRECTIONS: readonly DirectionFilter[] = ["all", "in", "out", "transfer"];
const SORTS: readonly TransactionSort[] = ["date_desc", "date_asc", "amount_desc", "amount_asc"];
const DEFAULT_KIND = "month";
const DEFAULT_SORT: TransactionSort = "date_desc";
const MAX_QUERY_LENGTH = 200;

const pick = <T extends string>(options: readonly T[], value: unknown): T | undefined => options.find((option) => option === value);

function positiveInt(value: unknown): number | undefined {
  const n = typeof value === "number" || typeof value === "string" ? Number(value) : Number.NaN;
  return Number.isInteger(n) && n > 0 ? n : undefined;
}

function accountIds(value: unknown): number[] {
  const list: unknown[] = Array.isArray(value) ? value : value === undefined ? [] : [value];
  return [...new Set(list.map(positiveInt).filter((id): id is number => id !== undefined))];
}

/** Turns whatever the URL held into a valid state; the URL is user-editable, so nothing is trusted. */
export function parseTransactionsSearch(raw: Record<string, unknown>): TransactionsSearch {
  const page = positiveInt(raw.page);
  const q = typeof raw.q === "string" || typeof raw.q === "number" ? String(raw.q).slice(0, MAX_QUERY_LENGTH) : "";
  return {
    ...parsePeriodSearch(raw, DEFAULT_KIND),
    accounts: accountIds(raw.account),
    q,
    direction: pick(DIRECTIONS, raw.direction) ?? "all",
    sort: pick(SORTS, raw.sort) ?? DEFAULT_SORT,
    ...(page && page > 1 && { page }),
  };
}

/** The URL form of a state: defaults are left out. */
export function transactionsParams(search: TransactionsSearch): Record<string, unknown> {
  return {
    ...periodParams(search, DEFAULT_KIND),
    ...(search.accounts.length > 0 && { account: search.accounts }),
    ...(search.q && { q: search.q }),
    ...(search.direction !== "all" && { direction: search.direction }),
    ...(search.sort !== DEFAULT_SORT && { sort: search.sort }),
    ...(search.page && search.page > 1 && { page: search.page }),
  };
}

/** The state back on its first page, which is where any change of filter, order or Period belongs. */
export function onFirstPage(search: TransactionsSearch): TransactionsSearch {
  const next = { ...search };
  delete next.page;
  return next;
}

/** True when an Account, the search or a direction narrows the Transactions (the Period alone does not count). */
export function hasActiveFilters(search: TransactionsSearch): boolean {
  return search.accounts.length > 0 || search.q.trim() !== "" || search.direction !== "all";
}

/** The state with no Account, search or direction; the Period and order stay. */
export function clearFilters(search: TransactionsSearch): TransactionsSearch {
  return onFirstPage({ ...search, accounts: [], q: "", direction: "all" });
}
