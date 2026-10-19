import { cn } from "../lib/utils";
import { Skeleton } from "./Skeleton";
import { skeletonWidth } from "./skeletonWidths";

interface TableSkeletonProps {
  /** What is loading, for assistive tech ("Loading Transactions"). */
  label: string;
  rows?: number;
  /** Cells per row; the first is the widest, like a description. */
  columns?: number;
  className?: string;
}

/** Placeholder rows shaped like a table's (`px-4 py-3`, a rule between them), so the real rows land where it was. */
export function TableSkeleton({ label, rows = 5, columns = 3, className }: TableSkeletonProps) {
  return (
    <div role="status" aria-busy="true" className={className}>
      <span className="sr-only">{label}</span>
      <ul>
        {Array.from({ length: rows }, (_, row) => (
          <li key={row} className="flex items-center gap-x-4 border-t border-border px-4 py-3 first:border-t-0">
            {Array.from({ length: columns }, (_, column) => (
              <Skeleton key={column} className={cn("h-4", column === 0 ? "flex-1" : "w-20", column === 0 && skeletonWidth(row, column))} />
            ))}
          </li>
        ))}
      </ul>
    </div>
  );
}
