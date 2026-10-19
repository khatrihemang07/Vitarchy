import { cn } from "../lib/utils";
import { Tooltip, TooltipContent, TooltipTrigger } from "../ui/tooltip";

/** One column heading, with an optional explanation shown on hover or focus. */
export interface ColumnHeading {
  id: string;
  label: string;
  tooltip?: string;
  /** Numbers line up to the end. */
  align?: "start" | "end";
}

interface ColumnHeaderBarProps {
  /** The row grid the rows share; it must switch to `md:grid`, because the bar shows only where the grid does. */
  grid: string;
  columns: readonly ColumnHeading[];
}

/** The muted header bar of a table built from grid rows. Rows stack on narrow screens, so the bar is hidden there. */
export function ColumnHeaderBar({ grid, columns }: ColumnHeaderBarProps) {
  return (
    <div className={cn(grid, "hidden gap-x-3 border-b border-border bg-muted/40 px-4 py-2 text-xs font-medium text-muted-foreground")}>
      {columns.map((column) => {
        const alignment = column.align === "end" ? "justify-self-end text-right" : undefined;
        if (!column.tooltip) {
          return (
            <span key={column.id} className={alignment}>
              {column.label}
            </span>
          );
        }
        return (
          <Tooltip key={column.id}>
            <TooltipTrigger asChild>
              <span
                tabIndex={0}
                className={cn("cursor-help rounded-sm underline decoration-dotted decoration-muted-foreground/50 underline-offset-4 focus-ring", alignment ?? "justify-self-start")}
              >
                {column.label}
              </span>
            </TooltipTrigger>
            <TooltipContent>{column.tooltip}</TooltipContent>
          </Tooltip>
        );
      })}
    </div>
  );
}
