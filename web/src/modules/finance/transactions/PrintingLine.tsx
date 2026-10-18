import type { PrintingItem } from "@/api/finance";
import { Badge, cn } from "@/design";
import { formatDateAndTime } from "../dates";
import { formatBalance } from "../money";

/** One Statement's Printing: its file, date, description, balance and reference, exactly as that Statement printed them. */
export function PrintingLine({ printing }: { printing: PrintingItem }) {
  const balance = formatBalance(printing.balance_paise);
  return (
    <li className={cn("flex flex-col gap-0.5 rounded-lg border bg-muted/30 px-3 py-2 text-xs", printing.gone && "opacity-70")}>
      <span className="flex flex-wrap items-center gap-x-2 gap-y-1">
        <span className="font-medium break-all">{printing.file_name}</span>
        <span className="text-muted-foreground">{formatDateAndTime(printing.date, printing.time)}</span>
        {printing.gone ? <Badge variant="outline" className="border-warning/50 text-warning">No longer found</Badge> : null}
      </span>
      <span className="[overflow-wrap:anywhere]">{printing.description}</span>
      {balance || printing.reference ? (
        <span className="flex flex-wrap gap-x-3 text-muted-foreground">
          {balance ? <span className="tabular-nums">Balance {balance}</span> : null}
          {printing.reference ? <span className="break-all">Ref {printing.reference}</span> : null}
        </span>
      ) : null}
    </li>
  );
}
