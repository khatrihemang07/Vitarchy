import type { SegmentView } from "@/api/time";
import { Badge } from "@/design";
import { formatSpan } from "../format";
import { type Clock, formatClock } from "../reports/entryTimes";
import { type AppColors, colorOf } from "./appColors";
import { domainOf, durationSecs, segmentLabel } from "./segmentRows";

interface SegmentRowProps {
  segment: SegmentView;
  colors: AppColors;
  clock: Clock;
}

/** One Segment of the Day: when, which app, its title and site, and how long. */
export function SegmentRow({ segment, colors, clock }: SegmentRowProps) {
  const domain = domainOf(segment.url);
  const details = [segment.title, domain].filter(Boolean).join(" · ");
  const away = segment.away === "yes";
  return (
    <li className="grid grid-cols-[1fr_auto] gap-x-3 gap-y-0.5 px-4 py-2 text-sm md:grid-cols-[9.5rem_17rem_minmax(0,1fr)_auto] md:items-baseline">
      <span className="text-muted-foreground tabular-nums">
        {formatClock(new Date(segment.start_at), clock)} – {formatClock(new Date(segment.end_at), clock)}
      </span>
      <span className="order-last col-span-2 flex min-w-0 items-center gap-2 md:order-none md:col-span-1">
        <span aria-hidden="true" className="size-2.5 shrink-0 rounded-full" style={{ background: away ? "var(--away)" : colorOf(colors, segment.app) }} />
        <span className="truncate font-medium">{segmentLabel(segment)}</span>
        {away ? <Badge variant="outline" className="text-muted-foreground">Away</Badge> : null}
        {segment.away === "unknown" ? <Badge variant="outline" className="text-muted-foreground">Away unknown</Badge> : null}
      </span>
      <span className="order-last col-span-2 min-w-0 truncate text-muted-foreground md:order-none md:col-span-1">{details}</span>
      <span className="text-right font-medium tabular-nums">{formatSpan(durationSecs(segment))}</span>
    </li>
  );
}
