import type { TransactionItem } from "@/api/finance";
import { Badge, cn } from "@/design";
import { AmountCell } from "../AmountCell";
import { formatDate } from "../dates";
import { formatBalance } from "../money";
import { PaymentRecordLine } from "./PaymentRecordLine";

interface TransactionRowProps {
  item: TransactionItem;
  /** The name of the Transaction's Account. */
  accountName: string;
}

/** One Transaction: date, Account, description with its linked Payment record, amount and balance. */
export function TransactionRow({ item, accountName }: TransactionRowProps) {
  return (
    <tr data-no-longer-found={item.no_longer_found} className={cn("border-t align-top", item.no_longer_found && "opacity-60")}>
      <td className="px-3 py-2 whitespace-nowrap">{formatDate(item.date)}</td>
      <td className="hidden px-3 py-2 text-muted-foreground sm:table-cell">{accountName}</td>
      <td className="px-3 py-2 [overflow-wrap:anywhere]">
        {item.description}
        <span className="block text-xs text-muted-foreground sm:hidden">{accountName}</span>
        {item.no_longer_found ? (
          <Badge variant="outline" className="ml-2 border-warning/50 text-warning">No longer found</Badge>
        ) : null}
        {item.payment_record ? <PaymentRecordLine record={item.payment_record} /> : null}
      </td>
      <td className="px-3 py-2 text-right whitespace-nowrap">
        <AmountCell paise={item.amount_paise} direction={item.direction} />
      </td>
      <td className="px-3 py-2 text-right whitespace-nowrap text-muted-foreground tabular-nums">
        {formatBalance(item.balance_paise)}
      </td>
    </tr>
  );
}
