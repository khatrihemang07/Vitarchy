import type { Ref } from "react";
import type { SegmentView } from "@/api/time";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/design";
import type { Clock } from "../reports/entryTimes";
import type { BandMark } from "./bandLayout";
import { markStyle } from "./markStyle";
import { SegmentTip } from "./SegmentTip";
import { segmentLabel, segmentWhen } from "./segmentRows";
import { describeAway } from "./timelineFormat";

interface BandMarkViewProps {
  mark: BandMark;
  /** The mark's main Segment. */
  segment: SegmentView;
  color: string;
  dimmed: boolean;
  clock: Clock;
  /** Only one mark is a Tab stop; the arrow keys move between marks. */
  tabbable: boolean;
  buttonRef: Ref<HTMLButtonElement>;
  onFocus: () => void;
}

/** One drawn mark of the band: a button whose tooltip (on hover and focus) describes its Segment. */
export function BandMarkView({ mark, segment, color, dimmed, clock, tabbable, buttonRef, onFocus }: BandMarkViewProps) {
  const more = mark.segments.length - 1;
  const label = [segmentWhen(segment, clock), segmentLabel(segment), segment.title, describeAway(segment.away)].filter(Boolean).join(", ");
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <button
          ref={buttonRef}
          type="button"
          tabIndex={tabbable ? 0 : -1}
          aria-label={more > 0 ? `${label}, and ${String(more)} other short Segments` : label}
          onFocus={onFocus}
          className="absolute inset-y-0 min-w-px outline-none transition-opacity focus-visible:z-10 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background"
          style={{ left: `${String(mark.left)}%`, width: `${String(mark.width)}%`, ...markStyle(segment.away, color, dimmed) }}
        />
      </TooltipTrigger>
      <TooltipContent><SegmentTip segment={segment} clock={clock} more={more} /></TooltipContent>
    </Tooltip>
  );
}
