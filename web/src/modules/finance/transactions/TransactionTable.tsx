import type { AccountSummary, TransactionItem } from "@/api/finance";
import { ColumnHeaderBar, SectionCard } from "@/design";
import { TRANSACTION_GRID, TRANSACTION_HEADINGS } from "../ledgerLayout";
import { TransactionRow } from "./TransactionRow";

/** The Transactions of one page, newest first, in the same titled card and header bar as Time's tables. */
export function TransactionTable({ items, accounts }: { items: readonly TransactionItem[]; accounts: readonly AccountSummary[] }) {
  const names = new Map(accounts.map((a) => [a.id, a.name]));
  return (
    <SectionCard title="Transactions">
      <ColumnHeaderBar grid={TRANSACTION_GRID} columns={TRANSACTION_HEADINGS} />
      <ul aria-label="Transactions" className="rounded-b-xl">
        {items.map((item) => (
          <TransactionRow key={item.id} item={item} accountName={names.get(item.account_id) ?? "Account"} names={names} />
        ))}
      </ul>
    </SectionCard>
  );
}
