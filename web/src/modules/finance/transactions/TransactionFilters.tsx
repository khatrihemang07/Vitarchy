import { useId } from "react";
import type { AccountSummary } from "@/api/finance";
import { ClearFiltersButton, Input, Label, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/design";
import { type TransactionsSearch, withFilter } from "../transactionsSearch";

const ALL = "all";

interface TransactionFiltersProps {
  search: TransactionsSearch;
  accounts: readonly AccountSummary[];
  onChange: (next: TransactionsSearch) => void;
}

/** Filters the Transactions by Account and by a range of dates, in the compact size Time's filters use. */
export function TransactionFilters({ search, accounts, onChange }: TransactionFiltersProps) {
  const id = useId();
  const filtered = search.account !== undefined || search.from !== undefined || search.to !== undefined;
  return (
    <div className="flex flex-wrap items-end gap-3" role="group" aria-label="Filters">
      <div className="flex flex-col gap-1">
        <Label id={`${id}-account`} className="text-xs text-muted-foreground">Account</Label>
        <Select
          value={search.account === undefined ? ALL : String(search.account)}
          onValueChange={(value) => { onChange(withFilter(search, { account: value === ALL ? undefined : Number(value) })); }}
        >
          <SelectTrigger size="sm" className="w-full sm:w-64" aria-labelledby={`${id}-account`}>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value={ALL}>All Accounts</SelectItem>
            {accounts.map((account) => (
              <SelectItem key={account.id} value={String(account.id)}>{account.name}</SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      <div className="flex flex-col gap-1">
        <Label htmlFor={`${id}-from`} className="text-xs text-muted-foreground">From</Label>
        <Input
          id={`${id}-from`}
          type="date"
          className="h-8 text-sm"
          value={search.from ?? ""}
          onChange={(event) => { onChange(withFilter(search, { from: event.target.value || undefined })); }}
        />
      </div>
      <div className="flex flex-col gap-1">
        <Label htmlFor={`${id}-to`} className="text-xs text-muted-foreground">To</Label>
        <Input
          id={`${id}-to`}
          type="date"
          className="h-8 text-sm"
          value={search.to ?? ""}
          onChange={(event) => { onChange(withFilter(search, { to: event.target.value || undefined })); }}
        />
      </div>
      {filtered ? <ClearFiltersButton onClear={() => { onChange({}); }} /> : null}
    </div>
  );
}
