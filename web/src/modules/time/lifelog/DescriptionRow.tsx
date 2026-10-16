import type { LifeLogDescriptionTotal } from "@/api/time";
import { cn } from "@/design";
import { formatCount } from "../format";
import { formatShare, formatSpan } from "../shared/duration";
import { CountsLine } from "./CountsLine";

interface DescriptionRowProps {
  total: LifeLogDescriptionTotal;
  /** All logged seconds in the Period, for the share. */
  loggedSeconds: number;
  /** Set when the row sits under a project. */
  nested?: boolean;
}

/** One description's row in the totals table. */
export function DescriptionRow({ total, loggedSeconds, nested = false }: DescriptionRowProps) {
  return (
    <tr className="border-t">
      <td className={cn("max-w-0 py-1.5 text-sm", nested ? "pl-9" : "pl-0")}>
        <span className="block truncate" title={total.description}>{total.description}</span>
        <CountsLine entries={total.entries} days={total.days} />
      </td>
      <td className="py-1.5 pl-3 text-right text-sm whitespace-nowrap tabular-nums">{formatSpan(total.seconds)}</td>
      <td className="hidden py-1.5 pl-3 text-right text-sm tabular-nums sm:table-cell">{formatCount(total.entries)}</td>
      <td className="hidden py-1.5 pl-3 text-right text-sm tabular-nums sm:table-cell">{formatCount(total.days)}</td>
      <td className="py-1.5 pl-3 text-right text-xs text-muted-foreground tabular-nums">{formatShare(total.seconds, loggedSeconds)}</td>
    </tr>
  );
}
