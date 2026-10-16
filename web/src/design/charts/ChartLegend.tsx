import { cn } from "../lib/utils";

/** One legend entry: a label and the colour of its series. */
export interface LegendItem {
  label: string;
  color: string;
}

interface ChartLegendProps {
  items: readonly LegendItem[];
  className?: string;
}

/** Names every series by colour. Charts with two or more series always show one. */
export function ChartLegend({ items, className }: ChartLegendProps) {
  return (
    <ul className={cn("flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground", className)}>
      {items.map((item, index) => (
        <li key={`${String(index)}-${item.label}`} className="flex items-center gap-1.5">
          <span aria-hidden className="size-2.5 rounded-sm" style={{ background: item.color }} />
          {item.label}
        </li>
      ))}
    </ul>
  );
}
