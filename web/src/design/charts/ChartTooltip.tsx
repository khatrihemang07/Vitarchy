import { cn } from "../lib/utils";

/** One row of a tooltip: a series, its colour and its formatted value. */
export interface TooltipRow {
  label: string;
  color: string;
  value: string;
}

interface ChartTooltipProps {
  /** What the hovered point is, e.g. a date. */
  title?: string;
  rows: readonly TooltipRow[];
  className?: string;
}

/** The hover card every chart uses. Text stays in ink; only the swatch carries the series colour. */
export function ChartTooltip({ title, rows, className }: ChartTooltipProps) {
  return (
    <div
      role="tooltip"
      className={cn(
        "min-w-32 rounded-md border bg-popover px-3 py-2 text-xs text-popover-foreground shadow-card",
        className,
      )}
    >
      {title ? <p className="mb-1 font-medium">{title}</p> : null}
      <ul className="flex flex-col gap-0.5">
        {rows.map((row) => (
          <li key={row.label} className="flex items-center gap-2">
            <span aria-hidden className="size-2 shrink-0 rounded-full" style={{ background: row.color }} />
            <span className="text-muted-foreground">{row.label}</span>
            <span className="ml-auto pl-3 font-medium tabular-nums">{row.value}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}
