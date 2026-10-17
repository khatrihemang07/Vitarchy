import type { ReactNode } from "react";

interface StatCellProps {
  /** Shown only when the row is stacked, since the table header is hidden then. */
  label: string;
  children: ReactNode;
}

/** One figure of a summary row; right-aligned digits on wide screens. */
export function StatCell({ label, children }: StatCellProps) {
  return (
    <span className="flex flex-col gap-0.5 text-sm tabular-nums md:items-end md:text-right">
      <span className="text-xs text-muted-foreground md:hidden">{label}</span>
      {children}
    </span>
  );
}
