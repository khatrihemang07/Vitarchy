import { useMemo, useRef, useState } from "react";
import type { SegmentView } from "@/api/time";
import type { Clock } from "../reports/entryTimes";
import { type AppColors, colorOf } from "./appColors";
import { BandMarkView } from "./BandMarkView";
import { BandTicks } from "./BandTicks";
import { dayBounds, hourTicks, layoutBand, nowPct } from "./bandLayout";
import { nextMark } from "./bandKeys";
import { matchesApp } from "./segmentRows";

interface DayBandProps {
  /** The local Day, `YYYY-MM-DD`. */
  day: string;
  segments: readonly SegmentView[];
  colors: AppColors;
  clock: Clock;
  /** The highlighted app; the other marks dim. */
  highlight?: string | undefined;
  now: Date;
  /** One sentence summarising the Day for screen readers. */
  summary: string;
}

/**
 * A 24 h track for one Day with a mark per Segment, coloured by app. Tab enters the band
 * on one mark and the arrow keys move between marks, each describing itself in a tooltip.
 */
export function DayBand({ day, segments, colors, clock, highlight, now, summary }: DayBandProps) {
  const bounds = useMemo(() => dayBounds(day), [day]);
  const marks = useMemo(() => layoutBand(segments, bounds), [segments, bounds]);
  const refs = useRef<(HTMLButtonElement | null)[]>([]);
  const [active, setActive] = useState(0);
  const current = Math.min(active, Math.max(0, marks.length - 1));
  const marker = nowPct(bounds, now.getTime());

  const onKeyDown = (event: React.KeyboardEvent) => {
    const next = nextMark(event.key, current, marks.length);
    if (next === null) return;
    event.preventDefault();
    setActive(next);
    refs.current[next]?.focus();
  };

  return (
    <div>
      <div
        role="toolbar"
        aria-label={summary}
        aria-orientation="horizontal"
        aria-describedby="band-help"
        onKeyDown={onKeyDown}
        className="relative h-14 overflow-hidden rounded-md border border-border bg-muted"
      >
        {hourTicks(day, 3).map(({ pct }) => (
          <span key={pct} aria-hidden="true" className="absolute inset-y-0 w-px bg-border" style={{ left: `${String(pct)}%` }} />
        ))}
        {marks.map((mark, index) => {
          const segment = segments[mark.main] as SegmentView;
          return (
            <BandMarkView
              key={mark.segments[0]}
              mark={mark}
              segment={segment}
              color={colorOf(colors, segment.app)}
              dimmed={highlight !== undefined && !matchesApp(segment, highlight)}
              clock={clock}
              tabbable={index === current}
              buttonRef={(el) => { refs.current[index] = el; }}
              onFocus={() => { setActive(index); }}
            />
          );
        })}
        {marker === null ? null : (
          <span aria-hidden="true" className="pointer-events-none absolute inset-y-0 z-20 w-0.5 bg-foreground" style={{ left: `${String(marker)}%` }} title="Now" />
        )}
      </div>
      <BandTicks day={day} clock={clock} />
      <p id="band-help" className="sr-only">Use the arrow keys to move between Segments.</p>
      {marker === null ? null : <p className="sr-only">The line marks now.</p>}
    </div>
  );
}
