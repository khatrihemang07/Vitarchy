import { useId, useState } from "react";
import type { TransactionItem } from "@/api/finance";
import { Badge, cn } from "@/design";
import { AmountCell } from "../AmountCell";
import { formatDate } from "../dates";
import { formatBalance } from "../money";
import { PaymentRecordLine } from "./PaymentRecordLine";
import { PrintingsList } from "./PrintingsList";
import { PrintingsToggle } from "./PrintingsToggle";

interface TransactionRowProps {
  item: TransactionItem;
  /** The name of the Transaction's Account. */
  accountName: string;
}

/** One Transaction: date, Account, description with its linked Payment record, amount and balance; expands to its Printings. */
export function TransactionRow({ item, accountName }: TransactionRowProps) {
  const [open, setOpen] = useState(false);
  const listId = useId();
  return (
    <>
      <tr data-no-longer-found={item.no_longer_found} className={cn("grid grid-cols-[1fr_auto] gap-x-3 border-t align-top sm:table-row", item.no_longer_found && "opacity-60")}>
        <td className="col-start-1 row-start-1 px-3 pt-2 whitespace-nowrap sm:py-2">{formatDate(item.date)}</td>
        <td className="col-start-1 row-start-2 px-3 text-xs text-muted-foreground sm:table-cell sm:py-2 sm:text-sm">{accountName}</td>
        <td className="col-span-2 row-start-3 min-w-0 px-3 py-2 [overflow-wrap:anywhere]">
          {item.description}
          {item.no_longer_found ? (
            <Badge variant="outline" className="ml-2 border-warning/50 text-warning">No longer found</Badge>
          ) : null}
          {item.payment_record ? <PaymentRecordLine record={item.payment_record} /> : null}
          {item.printing_count > 1 ? (
            <PrintingsToggle count={item.printing_count} open={open} controls={listId} onToggle={() => { setOpen((v) => !v); }} />
          ) : null}
        </td>
        <td className="col-start-2 row-span-2 row-start-1 px-3 pt-2 text-right whitespace-nowrap sm:table-cell sm:py-2">
          <AmountCell paise={item.amount_paise} direction={item.direction} />
          <span className="block text-xs text-muted-foreground tabular-nums sm:hidden">{formatBalance(item.balance_paise)}</span>
        </td>
        <td className="hidden px-3 py-2 text-right whitespace-nowrap text-muted-foreground tabular-nums sm:table-cell">
          {formatBalance(item.balance_paise)}
        </td>
      </tr>
      {open ? (
        <tr data-printings-for={item.id} className="block sm:table-row">
          <td colSpan={5} className="block px-3 pb-3 sm:table-cell">
            <PrintingsList transactionId={item.id} id={listId} />
          </td>
        </tr>
      ) : null}
    </>
  );
}
