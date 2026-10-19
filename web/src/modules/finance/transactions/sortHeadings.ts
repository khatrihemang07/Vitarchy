import type { ColumnHeading } from "@/design";
import { TRANSACTION_HEADINGS } from "../ledgerLayout";
import { orderOf, type SortColumn } from "./sortColumns";
import type { TransactionSort } from "./transactionsSearch";

const SORTABLE: readonly string[] = ["date", "amount"] satisfies SortColumn[];

/** The Transactions header bar with Date and Amount as sort buttons showing the current order. */
export function sortableHeadings(sort: TransactionSort, onSort: (column: SortColumn) => void): ColumnHeading[] {
  return TRANSACTION_HEADINGS.map((heading) => {
    if (!SORTABLE.includes(heading.id)) return heading;
    const column = heading.id as SortColumn;
    return { ...heading, sort: { order: orderOf(sort, column), onSort: () => { onSort(column); } } };
  });
}
