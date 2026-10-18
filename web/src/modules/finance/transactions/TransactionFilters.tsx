import { useId } from "react";
import type { AccountSummary } from "@/api/finance";
import { Input, Label, MotionButton, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/design";
import { type TransactionsSearch, withFilter } from "../transactionsSearch";

const ALL = "all";

interface TransactionFiltersProps {
  search: TransactionsSearch;
  accounts: readonly AccountSummary[];
  onChange: (next: TransactionsSearch) => void;
}

/** Filters the Transactions by Account and by a range of dates. */
export function TransactionFilters({ search, accounts, onChange }: TransactionFiltersProps) {
  const id = useId();
  const filtered = search.account !== undefined || search.from !== undefined || search.to !== undefined;
  return (
    <div className="flex flex-wrap items-end gap-3">
      <div className="flex flex-col gap-1.5">
        <Label id={`${id}-account`}>Account</Label>
        <Select
          value={search.account === undefined ? ALL : String(search.account)}
          onValueChange={(value) => { onChange(withFilter(search, { account: value === ALL ? undefined : Number(value) })); }}
        >
          <SelectTrigger className="w-full sm:w-64" aria-labelledby={`${id}-account`}>
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
      <div className="flex flex-col gap-1.5">
        <Label htmlFor={`${id}-from`}>From</Label>
        <Input
          id={`${id}-from`}
          type="date"
          value={search.from ?? ""}
          onChange={(event) => { onChange(withFilter(search, { from: event.target.value || undefined })); }}
        />
      </div>
      <div className="flex flex-col gap-1.5">
        <Label htmlFor={`${id}-to`}>To</Label>
        <Input
          id={`${id}-to`}
          type="date"
          value={search.to ?? ""}
          onChange={(event) => { onChange(withFilter(search, { to: event.target.value || undefined })); }}
        />
      </div>
      {filtered ? (
        <MotionButton variant="outline" onClick={() => { onChange({}); }}>
          Clear filters
        </MotionButton>
      ) : null}
    </div>
  );
}
