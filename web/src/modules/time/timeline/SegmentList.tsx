import { useState } from "react";
import type { SegmentView } from "@/api/time";
import type { Clock } from "../reports/entryTimes";
import type { AppColors } from "./appColors";
import { matchesApp } from "./segmentRows";
import { SegmentRow } from "./SegmentRow";

/** How many Segments show before "Show all"; a Day can run to hundreds. */
export const INITIAL_LIMIT = 50;

interface SegmentListProps {
  segments: readonly SegmentView[];
  colors: AppColors;
  clock: Clock;
  /** The highlighted app: only its Segments are listed. */
  highlight?: string | undefined;
}

/** The Day's Segments in time order: the first 50, then Show all; narrowed to the highlighted app. */
export function SegmentList({ segments, colors, clock, highlight }: SegmentListProps) {
  const [showAll, setShowAll] = useState(false);
  const matching = segments.filter((segment) => matchesApp(segment, highlight));
  const shown = showAll ? matching : matching.slice(0, INITIAL_LIMIT);
  if (matching.length === 0) {
    return <p className="px-4 py-3 text-sm text-muted-foreground">No Segments{highlight ? ` in ${highlight}` : ""} on this Day.</p>;
  }
  return (
    <div>
      <ul className="divide-y divide-border">
        {shown.map((segment) => (
          <SegmentRow key={`${segment.start_at}-${segment.app}`} segment={segment} colors={colors} clock={clock} />
        ))}
      </ul>
      {shown.length < matching.length ? (
        <button type="button" onClick={() => { setShowAll(true); }} className="w-full rounded-b-xl px-4 py-2 text-left text-sm font-medium text-module hover:underline focus-ring-inset">
          Show all {matching.length} Segments
        </button>
      ) : null}
    </div>
  );
}
