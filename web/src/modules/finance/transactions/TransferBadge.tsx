import type { TransferInfo } from "@/api/finance";
import { Badge, Popover, PopoverContent, PopoverTrigger } from "@/design";
import { TransferDetails } from "./TransferDetails";
import { transferBadgeLabel } from "./transferCopy";

interface TransferBadgeProps {
  transfer: TransferInfo;
  /** Whether this row is the Money out (it paid) or the Money in (it received). */
  direction: "debit" | "credit";
  /** Account names by id. */
  names: ReadonlyMap<number, string>;
}

/** "Transfer → ‹Account›" on a row; opens what is on the other side and Unlink. */
export function TransferBadge({ transfer, direction, names }: TransferBadgeProps) {
  return (
    <Popover>
      <PopoverTrigger asChild>
        <Badge asChild variant="info" className="cursor-pointer">
          <button type="button" aria-label={`${transferBadgeLabel(transfer, names)}, details`}>
            {transferBadgeLabel(transfer, names)}
          </button>
        </Badge>
      </PopoverTrigger>
      <PopoverContent className="flex flex-col gap-3 p-4">
        <TransferDetails transfer={transfer} direction={direction} names={names} />
      </PopoverContent>
    </Popover>
  );
}
