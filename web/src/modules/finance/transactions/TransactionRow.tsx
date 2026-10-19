import { useId, useState } from "react";
import type { TransactionItem } from "@/api/finance";
import { Badge, cn, Expand } from "@/design";
import { AmountCell } from "../AmountCell";
import { formatDateAndTime } from "../dates";
import { LEDGER_CELLS, LEDGER_ROW, TRANSACTION_GRID } from "../ledgerLayout";
import { formatBalance } from "../money";
import { RowActions } from "./RowActions";
import { PrintingsList } from "./PrintingsList";
import { PrintingsToggle } from "./PrintingsToggle";
import { TransferBadge } from "./TransferBadge";

interface TransactionRowProps {
  item: TransactionItem;
  /** The name of the Transaction's Account. */
  accountName: string;
  /** Account names by id, for naming the other side of a Transfer. */
  names: ReadonlyMap<number, string>;
}

/** One Transaction: date, Account, description, amount and balance; expands to its Printings. */
export function TransactionRow({ item, accountName, names }: TransactionRowProps) {
  const [open, setOpen] = useState(false);
  const listId = useId();
  const balance = formatBalance(item.balance_paise);
  return (
    <li data-no-longer-found={item.no_longer_found} className={cn("border-t border-border first:border-t-0 last:rounded-b-[inherit]", item.no_longer_found && "opacity-60")}>
      <div className={cn(LEDGER_ROW, TRANSACTION_GRID, "group/row transition-colors hover:bg-accent/50")}>
        <span className={LEDGER_CELLS.date}>{formatDateAndTime(item.date, item.time)}</span>
        <span className={LEDGER_CELLS.account}>{accountName}</span>
        <div className={LEDGER_CELLS.description}>
          {item.description}
          {item.no_longer_found ? <Badge variant="warning" className="ml-2">No longer found</Badge> : null}
          {item.transfer ? (
            <div className="mt-1"><TransferBadge transfer={item.transfer} direction={item.direction} names={names} /></div>
          ) : null}
          {item.printing_count > 1 ? (
            <PrintingsToggle count={item.printing_count} open={open} controls={listId} onToggle={() => { setOpen((v) => !v); }} />
          ) : null}
        </div>
        <span className={LEDGER_CELLS.amount}>
          <AmountCell paise={item.amount_paise} direction={item.direction} />
          <span className={LEDGER_CELLS.balanceUnderAmount}>{balance}</span>
        </span>
        <span className={LEDGER_CELLS.balance}>{balance}</span>
        <span className={LEDGER_CELLS.actions}>
          {item.transfer || item.no_longer_found ? null : <RowActions item={item} names={names} />}
        </span>
      </div>
      <Expand open={open} id={listId}>
        <div data-printings-for={item.id} className="border-t border-border bg-muted/20 px-4 py-3">
          <PrintingsList transactionId={item.id} />
        </div>
      </Expand>
    </li>
  );
}
