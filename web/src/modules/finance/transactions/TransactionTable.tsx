import type { AccountSummary, TransactionItem } from "@/api/finance";
import { TransactionRow } from "./TransactionRow";

/** The Transactions of one page, newest first. */
export function TransactionTable({ items, accounts }: { items: readonly TransactionItem[]; accounts: readonly AccountSummary[] }) {
  const names = new Map(accounts.map((a) => [a.id, a.name]));
  return (
    <div className="overflow-x-auto rounded-xl border bg-card">
      <table className="block w-full text-sm sm:table sm:min-w-[44rem]">
        <caption className="sr-only">Transactions</caption>
        <thead className="text-left max-sm:sr-only text-xs text-muted-foreground">
          <tr>
            <th scope="col" className="px-3 py-2 font-medium">Date</th>
            <th scope="col" className="hidden px-3 py-2 font-medium sm:table-cell">Account</th>
            <th scope="col" className="px-3 py-2 font-medium">Description</th>
            <th scope="col" className="px-3 py-2 text-right font-medium">Amount</th>
            <th scope="col" className="px-3 py-2 text-right font-medium">Balance</th>
          </tr>
        </thead>
        <tbody className="block sm:table-row-group">
          {items.map((item) => (
            <TransactionRow key={item.id} item={item} accountName={names.get(item.account_id) ?? "Account"} />
          ))}
        </tbody>
      </table>
    </div>
  );
}
