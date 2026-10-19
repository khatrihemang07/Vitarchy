import { useMemo } from "react";
import type { DayRange, WeekStart } from "@/lib/period/period";
import { periodHandlers } from "@/lib/period/periodHandlers";
import { type SortColumn, toggleSort } from "./sortColumns";
import { clearFilters, type DirectionFilter, onFirstPage, type TransactionSort, type TransactionsSearch } from "./transactionsSearch";

interface HandlerInput {
  search: TransactionsSearch;
  range: DayRange;
  today: string;
  weekStart: WeekStart;
  update: (next: TransactionsSearch) => void;
}

/** Every change the controls can make, expressed as a new URL state. Anything but paging goes back to the first page. */
export function useTransactionsHandlers({ search, range, today, weekStart, update }: HandlerInput) {
  return useMemo(() => {
    const change = (next: TransactionsSearch) => { update(onFirstPage(next)); };
    return {
      ...periodHandlers({ search, range, today, weekStart, update: change }),
      onAccounts: (ids: number[]) => { change({ ...search, accounts: ids }); },
      onSearch: (q: string) => { change({ ...search, q }); },
      onDirection: (direction: DirectionFilter) => { change({ ...search, direction }); },
      onSort: (sort: TransactionSort) => { change({ ...search, sort }); },
      onSortColumn: (column: SortColumn) => { change({ ...search, sort: toggleSort(search.sort, column) }); },
      onClear: () => { update(clearFilters(search)); },
      onPage: (page: number) => { update({ ...search, page }); },
    };
  }, [search, range, today, weekStart, update]);
}
