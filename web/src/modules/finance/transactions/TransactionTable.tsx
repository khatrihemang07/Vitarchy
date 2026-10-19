import type { AccountSummary, TransactionItem } from "@/api/finance";
import { ColumnHeaderBar, SectionCard } from "@/design";
import { TRANSACTION_GRID } from "../ledgerLayout";
import type { SortColumn } from "./sortColumns";
import { sortableHeadings } from "./sortHeadings";
import type { TransactionSort } from "./transactionsSearch";
import { TransactionRow } from "./TransactionRow";

interface TransactionTableProps {
  items: readonly TransactionItem[];
  accounts: readonly AccountSummary[];
  sort: TransactionSort;
  /** A Date or Amount heading was clicked. */
  onSort: (column: SortColumn) => void;
}

/** The Transactions of one page in the chosen order, in the same titled card and header bar as Time's tables. */
export function TransactionTable({ items, accounts, sort, onSort }: TransactionTableProps) {
  const names = new Map(accounts.map((a) => [a.id, a.name]));
  return (
    <SectionCard title="Transactions">
      <ColumnHeaderBar grid={TRANSACTION_GRID} columns={sortableHeadings(sort, onSort)} />
      <ul aria-label="Transactions" className="rounded-b-xl">
        {items.map((item) => (
          <TransactionRow key={item.id} item={item} accountName={names.get(item.account_id) ?? "Account"} names={names} />
        ))}
      </ul>
    </SectionCard>
  );
}
