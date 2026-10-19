import { Card } from "../ui/card";
import { Skeleton } from "./Skeleton";
import { skeletonWidth } from "./skeletonWidths";

interface CardSkeletonProps {
  /** What is loading, for assistive tech ("Loading Accounts"). */
  label: string;
  /** Lines of text under the title. */
  lines?: number;
}

/** A placeholder shaped like a titled Card of text, for pages that load whole cards. */
export function CardSkeleton({ label, lines = 3 }: CardSkeletonProps) {
  return (
    <Card role="status" aria-busy="true" className="gap-3 p-6">
      <span className="sr-only">{label}</span>
      <Skeleton className="h-5 w-1/3" />
      {Array.from({ length: lines }, (_, line) => (
        <Skeleton key={line} className={`h-4 ${skeletonWidth(line, 0)}`} />
      ))}
    </Card>
  );
}
