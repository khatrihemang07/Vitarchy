import { useState } from "react";
import { ChartLegend, ChartTooltip, OTHER_COLOR, SERIES_COUNT } from "@/design";
import { ChartTable } from "./ChartTable";
import { type Clock, formatHourOfDay, formatInstant } from "./timeOfDay";
import { colorLabels, placeSegments, segmentSeconds, TICK_HOURS, type TimelineSegment } from "./timelineLayout";

interface DayTimelineProps {
  /** The Day's first instant (inclusive) and last (exclusive), ISO. */
  dayStart: string;
  dayEnd: string;
  /** The IANA zone the times are shown in. */
  tz: string;
  clock: Clock;
  segments: readonly TimelineSegment[];
  /** Names what the lane shows, e.g. "Activity". */
  subject: string;
  /** Formats a segment's length for the tooltip and table. */
  formatSeconds: (seconds: number) => string;
}

const MIN_WIDTH = 0.15;

/** One Day as a single lane of coloured segments, with a legend, hover details and a table. Shared by the Activity and Life log tabs. */
export function DayTimeline({ dayStart, dayEnd, tz, clock, segments, subject, formatSeconds }: DayTimelineProps) {
  const [hovered, setHovered] = useState<number | null>(null);
  const placed = placeSegments(segments, dayStart, dayEnd);
  const legend = colorLabels(segments);
  const colors = new Map(legend.map((entry) => [entry.label, entry.color]));
  const named = legend.slice(0, SERIES_COUNT);
  const hasOther = legend.length > SERIES_COUNT;
  const active = hovered === null ? undefined : placed[hovered];
  const range = (segment: TimelineSegment) => `${formatInstant(segment.start, tz, clock)} – ${formatInstant(segment.end, tz, clock)}`;
  return (
    <div className="flex min-w-0 flex-col gap-3">
      <ChartLegend items={[...named.map((e) => ({ label: e.label, color: e.color })), ...(hasOther ? [{ label: "Other", color: OTHER_COLOR }] : [])]} />
      <div className="relative">
        <svg
          role="img"
          aria-label={`${subject} across the day, ${String(placed.length)} stretches. The same stretches follow as a table.`}
          className="block h-12 w-full rounded-md bg-muted"
          onMouseLeave={() => { setHovered(null); }}
        >
          {placed.map(({ segment, left, width }, index) => (
            <rect
              key={`${segment.start}-${segment.label}`}
              x={`${String(left)}%`}
              y={0}
              width={`${String(Math.max(width, MIN_WIDTH))}%`}
              height="100%"
              fill={colors.get(segment.label) ?? OTHER_COLOR}
              opacity={hovered === null || hovered === index ? 1 : 0.55}
              onMouseEnter={() => { setHovered(index); }}
            />
          ))}
        </svg>
        <div aria-hidden className="relative mt-1 h-4 text-xs text-muted-foreground">
          {TICK_HOURS.map((hour) => (
            <span
              key={hour}
              className="absolute whitespace-nowrap"
              style={{ left: `${String((hour / 24) * 100)}%`, transform: hour === 0 ? "none" : hour === 24 ? "translateX(-100%)" : "translateX(-50%)" }}
            >
              {formatHourOfDay(hour, clock)}
            </span>
          ))}
        </div>
        {active ? (
          <div className="pointer-events-none absolute top-12 z-10 -translate-x-1/2" style={{ left: `${String(Math.min(Math.max(active.left + active.width / 2, 12), 88))}%` }}>
            <ChartTooltip
              title={active.segment.label}
              rows={[
                ...(active.segment.detail ? [{ label: "In", color: colors.get(active.segment.label) ?? OTHER_COLOR, value: active.segment.detail }] : []),
                { label: range(active.segment), color: colors.get(active.segment.label) ?? OTHER_COLOR, value: formatSeconds(segmentSeconds(active.segment)) },
              ]}
            />
          </div>
        ) : null}
      </div>
      <ChartTable
        caption={`${subject} across the day`}
        columns={["Time", "What", "Length"]}
        rows={placed.map(({ segment }) => [range(segment), segment.detail ? `${segment.label} (${segment.detail})` : segment.label, formatSeconds(segmentSeconds(segment))])}
      />
    </div>
  );
}
