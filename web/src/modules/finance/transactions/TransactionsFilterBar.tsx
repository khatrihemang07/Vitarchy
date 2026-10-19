import type { AccountSummary } from "@/api/finance";
import {
  ClearFiltersButton,
  DescriptionSearch,
  FacetFilter,
  PeriodPicker,
  SegmentedControl,
  type SegmentedOption,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/design";
import { containsDay, type DayRange } from "@/lib/period/period";
import { SORT_LABELS } from "./sortColumns";
import { hasActiveFilters, type TransactionSort, type TransactionsSearch } from "./transactionsSearch";
import type { useTransactionsHandlers } from "./useTransactionsHandlers";

const DIRECTIONS: readonly SegmentedOption[] = [
  { value: "all", label: "All" },
  { value: "in", label: "Money in" },
  { value: "out", label: "Money out" },
  { value: "transfer", label: "Transfers" },
];

interface FilterBarProps {
  search: TransactionsSearch;
  range: DayRange;
  today: string;
  /** Every Account; hidden ones are left out of the picker. */
  accounts: readonly AccountSummary[];
  handlers: ReturnType<typeof useTransactionsHandlers>;
}

const SORTS = Object.keys(SORT_LABELS) as TransactionSort[];

/** The Period, the Accounts, a description search and a direction, with one Clear filters. */
export function TransactionsFilterBar({ search, range, today, accounts, handlers }: FilterBarProps) {
  const options = accounts.filter((a) => !a.hidden).map((a) => ({ value: String(a.id), label: a.name }));
  return (
    <div className="flex flex-col gap-3">
      <PeriodPicker
        kind={search.kind}
        range={range}
        isCurrent={search.kind !== "custom" && containsDay(range, today)}
        onKind={handlers.onKind}
        onShift={handlers.onShift}
        onToday={handlers.onToday}
        onCustom={handlers.onCustom}
      />
      <div className="flex flex-wrap items-center gap-2" role="group" aria-label="Filters">
        <FacetFilter label="Account" options={options} selected={search.accounts.map(String)} onChange={(values) => { handlers.onAccounts(values.map(Number)); }} />
        <DescriptionSearch value={search.q} onSearch={handlers.onSearch} />
        <SegmentedControl label="Direction" value={search.direction} options={DIRECTIONS} onValueChange={(value) => { handlers.onDirection(value as TransactionsSearch["direction"]); }} />
        {/* Narrow screens have no header bar to sort by, so the order is picked here. */}
        <Select value={search.sort} onValueChange={(value) => { handlers.onSort(value as TransactionSort); }}>
          <SelectTrigger size="sm" aria-label="Sort by" className="w-40 md:hidden">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {SORTS.map((sort) => (
              <SelectItem key={sort} value={sort}>{SORT_LABELS[sort]}</SelectItem>
            ))}
          </SelectContent>
        </Select>
        {hasActiveFilters(search) ? <ClearFiltersButton placement="bar" onClear={handlers.onClear} /> : null}
      </div>
    </div>
  );
}
