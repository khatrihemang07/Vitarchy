import { ChevronRight } from "lucide-react";
import { type ReactNode, useState } from "react";
import { cn } from "@/design";

interface BarRowProps {
  label: string;
  /** Formatted length of time. */
  value: string;
  /** Formatted share of the total. */
  share: string;
  /** Bar length, 0 to 1, against the longest sibling. */
  fraction: number;
  color: string;
  /** Rows nested under this one; their presence makes the row expandable. */
  children?: ReactNode;
  /** 0 for an app, 1 for a Context, 2 for a title. */
  level?: number;
}

/** One row of a bar list: name, a thin bar, the time and its share. Expands to show its children. */
export function BarRow({ label, value, share, fraction, color, children, level = 0 }: BarRowProps) {
  const [open, setOpen] = useState(false);
  const expandable = children !== undefined;
  const content = (
    <>
      <span className="flex min-w-0 items-center gap-1.5">
        {expandable ? (
          <ChevronRight aria-hidden className={cn("size-4 shrink-0 text-muted-foreground transition-transform motion-reduce:transition-none", open && "rotate-90")} />
        ) : (
          <span aria-hidden className="size-4 shrink-0" />
        )}
        <span className={cn("truncate", level === 0 ? "font-medium" : "text-sm")} title={label}>
          {label}
        </span>
      </span>
      <span className="ml-auto shrink-0 pl-3 text-sm tabular-nums">{value}</span>
      <span className="w-10 shrink-0 text-right text-xs text-muted-foreground tabular-nums">{share}</span>
    </>
  );
  const bar = (
    <span aria-hidden className="mt-1 block h-1.5 rounded-full bg-muted">
      <span className="block h-full rounded-full" style={{ width: `${String(Math.max(fraction * 100, 1))}%`, background: color }} />
    </span>
  );
  return (
    <li className={cn(level > 0 && "pl-5")}>
      {expandable ? (
        <button
          type="button"
          aria-expanded={open}
          onClick={() => { setOpen(!open); }}
          className="w-full rounded-md py-1.5 text-left hover:bg-accent/50 focus-visible:outline-2 focus-visible:outline-ring"
        >
          <span className="flex items-baseline">{content}</span>
          {bar}
        </button>
      ) : (
        <div className="py-1.5">
          <span className="flex items-baseline">{content}</span>
          {bar}
        </div>
      )}
      {expandable && open ? <ul className="mt-1 flex flex-col">{children}</ul> : null}
    </li>
  );
}
