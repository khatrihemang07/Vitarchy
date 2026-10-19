import { ArrowDown, ArrowUp, ChevronsUpDown } from "lucide-react";
import { cn } from "../lib/utils";
import { Tooltip, TooltipContent, TooltipTrigger } from "../ui/tooltip";

/** How a column is ordered now (null when the table is ordered by another column) and what a click does. */
export interface SortState {
  order: "asc" | "desc" | null;
  onSort: () => void;
}

const ARIA_SORT = { asc: "ascending", desc: "descending", none: "none" } as const;
const ICONS = { asc: ArrowUp, desc: ArrowDown } as const;

interface SortButtonProps {
  label: string;
  tooltip: string | undefined;
  sort: SortState;
  className?: string | undefined;
}

/** A column heading that sorts by its column when clicked; `aria-sort` and an arrow say how it is ordered. */
export function SortButton({ label, tooltip, sort, className }: SortButtonProps) {
  const Icon = sort.order ? ICONS[sort.order] : ChevronsUpDown;
  const button = (
    <button
      type="button"
      onClick={sort.onSort}
      className={cn("flex items-center gap-1 rounded-sm font-medium hover:text-foreground focus-ring", sort.order && "text-foreground")}
    >
      {label}
      <Icon className={cn("size-3.5", !sort.order && "opacity-50")} aria-hidden="true" />
    </button>
  );
  return (
    // aria-sort belongs on a column header, not on the button inside it.
    <span role="columnheader" aria-sort={ARIA_SORT[sort.order ?? "none"]} className={className}>
      {tooltip ? (
        <Tooltip>
          <TooltipTrigger asChild>{button}</TooltipTrigger>
          <TooltipContent>{tooltip}</TooltipContent>
        </Tooltip>
      ) : (
        button
      )}
    </span>
  );
}
