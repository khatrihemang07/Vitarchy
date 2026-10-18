import { cn } from "@/design";

import { AwayFigure } from "./AwayFigure";
import type { AwayPreviewRow } from "./awayPreviewRows";

/** One threshold of the Away rules table; the one in force is highlighted. */
export function AwayChoiceRow({ row }: { row: AwayPreviewRow }) {
  return (
    <tr aria-current={row.current ? "true" : undefined} className={cn("border-t border-border", row.current && "bg-module/15")}>
      <th scope="row" className="px-2 py-2 text-left font-medium">
        {row.label}
      </th>
      <AwayFigure value={row.away} change={row.awayChange} />
      <AwayFigure value={row.active} change={row.activeChange} />
    </tr>
  );
}
