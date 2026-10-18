import type { TimelineSummary } from "@/api/time";
import { FadeUp } from "@/design";
import type { Clock } from "../reports/entryTimes";
import { bodySections } from "./bodySections";
import { EmptyTimeline } from "./EmptyTimeline";
import { hasTimeline } from "./overviewFigures";

interface TimelineBodyProps {
  summary: TimelineSummary;
  clock: Clock;
  now: Date;
  highlight?: string | undefined;
  onApp: (app: string | undefined) => void;
  /** Place in the page's entrance sequence of the first section. */
  firstStagger: number;
}

/** A loaded summary: its sections in order, each fading up in turn, or the empty notice. */
export function TimelineBody({ summary, clock, now, highlight, onApp, firstStagger }: TimelineBodyProps) {
  if (!hasTimeline(summary)) {
    return (
      <FadeUp index={firstStagger}>
        <EmptyTimeline />
      </FadeUp>
    );
  }
  return (
    <div className="flex flex-col gap-6">
      {bodySections({ summary, clock, now, highlight, onApp }).map(({ key, node }, index) => (
        <FadeUp key={key} index={firstStagger + index}>{node}</FadeUp>
      ))}
    </div>
  );
}
