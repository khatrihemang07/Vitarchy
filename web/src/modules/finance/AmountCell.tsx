import type { Direction } from "@/api/finance";
import { cn } from "@/design";
import { formatSignedAmount, isCredit } from "./money";

/** A Transaction's amount in INR, coloured by direction: credits in the success tone, debits in the destructive one. */
export function AmountCell({ paise, direction }: { paise: number; direction: Direction }) {
  return (
    <span
      data-direction={direction}
      className={cn("font-medium tabular-nums", isCredit(direction) ? "text-success" : "text-destructive")}
    >
      {formatSignedAmount(paise, direction)}
    </span>
  );
}
