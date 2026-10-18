import type { SegmentView } from "@/api/time";
import type { Clock } from "../reports/entryTimes";
import { segmentLabel, segmentWhen } from "./segmentRows";
import { describeAway } from "./timelineFormat";

/** What a Segment's tooltip says: when, which app and window, and whether you were away. */
export function SegmentTip({ segment, clock, more }: { segment: SegmentView; clock: Clock; more: number }) {
  return (
    <div className="flex max-w-64 flex-col gap-0.5 break-words">
      <span className="tabular-nums">{segmentWhen(segment, clock)}</span>
      <span className="font-medium">{segmentLabel(segment)}</span>
      {segment.title ? <span className="line-clamp-3 opacity-80">{segment.title}</span> : null}
      <span className="opacity-80">{describeAway(segment.away)}</span>
      {more > 0 ? <span className="opacity-80">and {more} other short {more === 1 ? "Segment" : "Segments"}</span> : null}
    </div>
  );
}
