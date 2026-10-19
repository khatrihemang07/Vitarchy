import type { TransactionSort } from "./transactionsSearch";

/** The columns the table can be sorted by. */
export type SortColumn = "date" | "amount";

/** How a sort reads: which column and which way. */
export interface SortReading {
  column: SortColumn;
  order: "asc" | "desc";
}

/** The column and direction a sort value means. */
export function readSort(sort: TransactionSort): SortReading {
  const [column, order] = sort.split("_") as [SortColumn, "asc" | "desc"];
  return { column, order };
}

/** The sort after clicking `column`: its other direction when it is already the sort, else newest or largest first. */
export function toggleSort(sort: TransactionSort, column: SortColumn): TransactionSort {
  const now = readSort(sort);
  const order = now.column === column && now.order === "desc" ? "asc" : "desc";
  return `${column}_${order}`;
}

/** The order a column shows: its direction while it is the sort, null otherwise. */
export function orderOf(sort: TransactionSort, column: SortColumn): "asc" | "desc" | null {
  const now = readSort(sort);
  return now.column === column ? now.order : null;
}

/** The words of each order, for the picker shown where there is no header bar (narrow screens). */
export const SORT_LABELS: Record<TransactionSort, string> = {
  date_desc: "Newest first",
  date_asc: "Oldest first",
  amount_desc: "Largest first",
  amount_asc: "Smallest first",
};
