import type { TransactionView } from "@/api/finance";
import { Badge, cn, ColumnHeaderBar, SectionCard } from "@/design";
import { AmountCell } from "./AmountCell";
import { formatDateAndTime } from "./dates";
import { LEDGER_CELLS, LEDGER_ROW, STATEMENT_ROW_GRID, STATEMENT_ROW_HEADINGS } from "./ledgerLayout";
import { formatBalance } from "./money";

/** The rows of a Statement section: date, description, amount and balance. Rows already stored are dimmed. */
export function StatementRowsTable({ rows, caption }: { rows: readonly TransactionView[]; caption: string }) {
  return (
    <SectionCard title="Transactions">
      <ColumnHeaderBar grid={STATEMENT_ROW_GRID} columns={STATEMENT_ROW_HEADINGS} />
      <ul aria-label={caption} className="max-h-96 overflow-y-auto rounded-b-xl">
        {rows.map((row, index) => {
          const balance = formatBalance(row.balance_paise);
          return (
            <li
              key={index}
              data-already-stored={row.is_already_stored}
              className={cn(LEDGER_ROW, STATEMENT_ROW_GRID, "border-t border-border transition-colors first:border-t-0 hover:bg-accent/50", row.is_already_stored && "opacity-50")}
            >
              <span className={LEDGER_CELLS.date}>{formatDateAndTime(row.date, row.time)}</span>
              <div className={LEDGER_CELLS.description}>
                {row.description}
                {row.is_already_stored ? <span className="ml-2 text-xs text-muted-foreground">Already stored</span> : null}
                {row.gains_detail ? <Badge variant="info" className="ml-2">Gains detail</Badge> : null}
              </div>
              <span className={LEDGER_CELLS.amount}>
                <AmountCell paise={row.amount_paise} direction={row.direction} />
                <span className={LEDGER_CELLS.balanceUnderAmount}>{balance}</span>
              </span>
              <span className={LEDGER_CELLS.balance}>{balance}</span>
            </li>
          );
        })}
      </ul>
    </SectionCard>
  );
}
