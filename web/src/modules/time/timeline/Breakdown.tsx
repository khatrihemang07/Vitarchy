import { formatSpan } from "../format";

interface BreakdownProps {
  heading: string;
  rows: { name: string; secs: number }[];
}

/** A small headed list of names with their Active time; renders nothing without rows. */
export function Breakdown({ heading, rows }: BreakdownProps) {
  if (rows.length === 0) return null;
  return (
    <div className="min-w-0">
      <h3 className="mb-1 text-xs font-medium text-muted-foreground">{heading}</h3>
      <ul className="flex flex-col gap-0.5 text-sm">
        {rows.map((row) => (
          <li key={row.name} className="flex items-baseline justify-between gap-3">
            <span className="min-w-0 truncate">{row.name || "(untitled)"}</span>
            <span className="shrink-0 tabular-nums text-muted-foreground">{formatSpan(row.secs)}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}
