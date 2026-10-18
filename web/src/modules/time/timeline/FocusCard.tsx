import type { TimelineOverview } from "@/api/time";
import { formatSpan } from "../format";
import type { Clock } from "../reports/entryTimes";
import { formatMoment, formatPerHour, formatShare } from "./timelineFormat";

interface Figure {
  label: string;
  value: string;
  note: string;
}

function figuresOf(overview: TimelineOverview, clock: Clock, isDay: boolean): Figure[] {
  return [
    { label: "Longest Focus stretch", value: formatSpan(overview.longest_focus_secs), note: "Time in one app without a real break; a glance away under a minute does not end it." },
    { label: "Focused share", value: formatShare(overview.focus_share), note: "Share of Active time spent in Focus stretches of 25 minutes or more." },
    { label: "Switches per Active hour", value: formatPerHour(overview.switches_per_active_hour), note: "How often you moved from one app to another." },
    { label: isDay ? "First Active" : "Earliest Active", value: formatMoment(overview.first_active, clock, !isDay), note: "When the first Active moment began." },
    { label: isDay ? "Last Active" : "Latest Active", value: formatMoment(overview.last_active, clock, !isDay), note: "When the last Active moment ended." },
  ];
}

/** Focus and Switching, and when the Active time started and ended, each with a one-line meaning. */
export function FocusCard({ overview, clock, isDay }: { overview: TimelineOverview; clock: Clock; isDay: boolean }) {
  return (
    <dl className="flex flex-col divide-y divide-border">
      {figuresOf(overview, clock, isDay).map((figure) => (
        <div key={figure.label} className="flex flex-col gap-0.5 py-2 first:pt-0 last:pb-0">
          <div className="flex items-baseline justify-between gap-3">
            <dt className="text-sm text-muted-foreground">{figure.label}</dt>
            <dd className="font-display text-lg font-bold tabular-nums">{figure.value}</dd>
          </div>
          <p className="text-xs text-muted-foreground">{figure.note}</p>
        </div>
      ))}
    </dl>
  );
}
