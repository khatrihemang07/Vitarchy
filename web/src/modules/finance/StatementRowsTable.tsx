import type { TransactionView } from "@/api/finance";
import { cn } from "@/design";
import { AmountCell } from "./AmountCell";
import { formatDate } from "./dates";
import { formatBalance } from "./money";

/** The rows of a Statement section: date, description, amount and balance. Rows already stored are dimmed. */
export function StatementRowsTable({ rows, caption }: { rows: readonly TransactionView[]; caption: string }) {
  return (
    <div className="max-h-96 overflow-auto rounded-lg border">
      <table className="w-full sm:min-w-[34rem] text-sm">
        <caption className="sr-only">{caption}</caption>
        <thead className="sticky top-0 bg-card text-left text-xs text-muted-foreground">
          <tr>
            <th scope="col" className="hidden px-3 py-2 font-medium sm:table-cell">Date</th>
            <th scope="col" className="px-3 py-2 font-medium">Description</th>
            <th scope="col" className="px-3 py-2 text-right font-medium">Amount</th>
            <th scope="col" className="hidden px-3 py-2 text-right font-medium sm:table-cell">Balance</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row, index) => {
            const balance = formatBalance(row.balance_paise);
            return (
            <tr key={index} data-duplicate={row.is_duplicate} className={cn("border-t", row.is_duplicate && "opacity-50")}>
              <td className="hidden px-3 py-2 whitespace-nowrap sm:table-cell">{formatDate(row.date)}</td>
              <td className="min-w-0 px-3 py-2 [overflow-wrap:anywhere]">
                <span className="block text-xs text-muted-foreground sm:hidden">{formatDate(row.date)}</span>
                {row.description}
                {row.is_duplicate ? <span className="ml-2 text-xs text-muted-foreground">Already stored</span> : null}
              </td>
              <td className="px-3 py-2 text-right whitespace-nowrap">
                <AmountCell paise={row.amount_paise} direction={row.direction} />
                <span className="block text-xs text-muted-foreground tabular-nums sm:hidden">{balance}</span>
              </td>
              <td className="hidden px-3 py-2 text-right whitespace-nowrap text-muted-foreground tabular-nums sm:table-cell">{balance}</td>
            </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
